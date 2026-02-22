use std::{path::PathBuf, sync::Arc};

use clap::{Parser};
use anyhow::{Context, Result};
use tokio::process::Command;
use crate::{cmd::ssh::utils::ParsedSshArgs, config::appconfig::{AppConfig, ChannelOptions,
    Ssh, ShellOptions}, executor::{Executor, MultiplexedSession},
    remote_shell::{sh::ShRemoteShell, Channel, Shell}};
pub mod controlpath;
pub mod utils;
use rand::Rng;
use log::debug;
use log::error;

pub mod lchannel;
pub mod bootstrap;

const REMOTE_WORKDIR: &str = "/tmp";

#[derive(Parser)]
pub struct SshCmd {
    #[arg(trailing_var_arg = true, allow_hyphen_values=true)]
    pub ssh_args: Vec<String>,
}

impl SshCmd {
    pub async fn exec(&self,
        executor: Arc<Executor>,
        prompts: Vec<String>,
        session_info: MultiplexedSession,
        appconfig: &AppConfig,
        parsed_ssh_args: ParsedSshArgs
    )-> Result<()> {

        let usock_path = PathBuf::from(&session_info.controlpath)
            .parent()
            .context("Error getting control path's parent dir")?
            .join("pcmd.sock");

        let usock_path_str = usock_path.to_string_lossy().to_string();

        let remote_default = Ssh::default();
        let remote_config = appconfig.find_ssh_best_match(
            &session_info.destination.hostname_for_match,
            session_info.destination.username.as_deref())
            .unwrap_or(&remote_default);

        let mut rng = rand::rng();
        let rand_suffix = rng.random_range(u32::MIN..u32::MAX).to_string();

        let remote_workdir = PathBuf::from(REMOTE_WORKDIR)
            .join(format!("pcmd_{rand_suffix}"))
            .to_string_lossy().to_string();

        let shell = match remote_config.shell {
            ShellOptions::Auto => Shell::Auto(remote_workdir.clone()),
            ShellOptions::Bash => Shell::Bash(remote_config.bash_method.clone(), remote_workdir.clone()),
            ShellOptions::Zsh => Shell::Zsh(remote_workdir.clone()),
            ShellOptions::Sh => Shell::Sh(remote_workdir.clone()),
            ShellOptions::Ash => Shell::Ash(remote_workdir.clone()),
            ShellOptions::Dash => Shell::Dash(remote_workdir.clone()),
            ShellOptions::Fish => Shell::Fish(remote_workdir.clone())
        };

        debug!("Using shell: {:#?}", shell);

        let local_port = rng.random_range(remote_config.local_ports.start..=remote_config.local_ports.end);
        let remote_port = rng.random_range(remote_config.remote_ports.start..=remote_config.remote_ports.end);

        let channel = match remote_config.channel {
            ChannelOptions::Auto  => {
                Channel::Nc(local_port, remote_port)
            },
            ChannelOptions::Nc => {
                Channel::Nc(local_port, remote_port)
            },
            ChannelOptions::Socat => {
                let remote_socket_filename = format!("pcmd_{rand_suffix}.sock");
                let remote_socket = PathBuf::from(&remote_config.remote_socket.path)
                    .join(remote_socket_filename);

                Channel::Socat(usock_path_str, remote_socket.to_string_lossy().to_string())
            },
            ChannelOptions::BashTcp => {
                Channel::BashTcp(local_port, remote_port)
            },
            ChannelOptions::Fifo  => {
                Channel::Fifo(remote_workdir.clone())
            },
            ChannelOptions::FifoSingle  => {
                Channel::FifoSingle(remote_workdir.clone())
            },
        };

        debug!("Using channel: {:#?}", channel);

        let remote_cmd = if parsed_ssh_args.server_args.len() > 1 {
            Some(parsed_ssh_args.server_args[1..].join(" "))
        } else { None };

        let bootstrap_data = bootstrap::setup(executor, &channel)?;
        let bootstrap_script = ShRemoteShell::bootstrap("sh", remote_workdir.as_str(), "dispatch", &prompts, &channel, &shell, &remote_config.bash_method, remote_cmd.as_deref());

        tokio::spawn(async move {
            let res = bootstrap_data.lchannel.run().await;
            if let Err(err) = res {
                error!("Channel error: {err}");
            }
            Ok::<(), anyhow::Error>(())
        });

        let connection_sharing_args = [
            "-o".to_string(),
            "ControlMaster=yes".to_string(),
            "-o".to_string(),
            format!("ControlPath={}", &session_info.controlpath),
            "-o".to_string(),
            "ControlPersist=no".to_string(),
        ];

        debug!("SSH Args: {:?}", &parsed_ssh_args.ssh_args);
        debug!("Server Args: {:?}", &parsed_ssh_args.server_args);

        let ssh_cmd_handle = tokio::spawn(async move {

        let initial_args = if remote_cmd.is_none() {
            vec![String::from("-t")]
        } else {
            vec![String::from("-n")]
        };

        let forwards: Vec<String> = bootstrap_data.forwards.iter()
                    .map(|f| format!("-R {}:{}", f.remote, f.local)).collect();

        let full_args: Vec<&str> = initial_args.iter()
            .chain(forwards.iter())
            .chain(connection_sharing_args.iter())
            .chain(parsed_ssh_args.ssh_args.iter())
            .chain(std::iter::once(&parsed_ssh_args.server_args[0]))
            .chain(std::iter::once(&bootstrap_script))
            .map(|s| s.as_str())
            .collect();

            // println!("ssh {}", full_args.join(" "));
            debug!("{}", &bootstrap_script);

            Command::new("ssh")
                .args(full_args)
                .spawn().context("Error spawning ssh")?
                .wait().await.context("Error in ssh proc")?;
            Ok::<(), anyhow::Error>(())
        });

        ssh_cmd_handle.await??;

        Ok(())
    }
}
