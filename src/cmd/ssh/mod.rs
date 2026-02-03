use std::{path::PathBuf, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use clap::{Parser};
use anyhow::{Context, Result};
use tokio::process::Command;
use crate::{cmd::ssh::{shell::{Channel, Shell}, utils::ParsedSshArgs}, config::appconfig::{AppConfig, ChannelOptions, Remote, ShellOptions}, executor::{Executor, MultiplexedSession}};
pub mod controlpath;
pub mod utils;
use rand::Rng;
use log::debug;

pub mod shell;
pub mod lchannel;
pub mod bootstrap;

const REMOTE_WORKDIR: &str = "/tmp/pcmd";

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

        let controlpath = session_info.controlpath;
        let usock_path = PathBuf::from(&controlpath)
            .parent()
            .context("Error getting control path's parent dir")?
            .join("pcmd.sock");

        let usock_path_str = usock_path.to_string_lossy().to_string();

        let remote_default = Remote::default();
        let remote_config = appconfig.find_remote_best_match(
            &session_info.destination.hostname_for_match,
            session_info.destination.username.as_deref())
            .unwrap_or(&remote_default);

        let shell = match remote_config.shell {
            ShellOptions::Auto => Shell::Auto(REMOTE_WORKDIR),
            ShellOptions::Bash => Shell::Bash,
            ShellOptions::Zsh => Shell::Zsh(REMOTE_WORKDIR),
            ShellOptions::Sh => Shell::Sh(REMOTE_WORKDIR),
            ShellOptions::Fish => Shell::Fish(REMOTE_WORKDIR)
        };

        let mut rng = rand::rng();
        let local_port = rng.random_range(remote_config.local_ports.start..=remote_config.local_ports.end);
        let remote_port = rng.random_range(remote_config.remote_ports.start..=remote_config.remote_ports.end);

        let channel = match remote_config.channel {
            ChannelOptions::Nc => {
                Channel::Nc(local_port, remote_port)
            },
            ChannelOptions::Socat => {
                let time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)?
                    .as_secs().to_string();

                let remote_socket_filename = format!("pcmd_{time}.sock");
                let remote_socket = PathBuf::from(&remote_config.remote_socket.path)
                    .join(remote_socket_filename);

                Channel::Socat(usock_path_str, remote_socket.to_string_lossy().to_string())
            },
            ChannelOptions::BashTcp => {
                Channel::BashTcp(local_port, remote_port)
            },
            _ => {
                Channel::Nc(local_port, remote_port)
            }
        };

        let remote_cmd = if parsed_ssh_args.server_args.len() > 1 {
            Some(&parsed_ssh_args.server_args[1..].iter().map(|s| s.as_str()).collect::<Vec<_>>()[..])
        } else { None };
        let bootstrap_data = bootstrap::setup(executor, &prompts, shell, channel, remote_cmd)?;

        tokio::spawn(async move {
            bootstrap_data.lchannel.run().await.context("Channel Error")?;
            Ok::<(), anyhow::Error>(())
        });


        let connection_sharing_args = [
            "-o".to_string(),
            "ControlMaster=yes".to_string(),
            "-o".to_string(),
            format!("ControlPath={}", &controlpath),
            "-o".to_string(),
            "ControlPersist=no".to_string(),
        ];

        debug!("SSH Args: {:?}", &parsed_ssh_args.ssh_args);
        debug!("Server Args: {:?}", &parsed_ssh_args.server_args);


        let ssh_cmd_handle = tokio::spawn(async move {

        let initial_args = [String::from("-t"), String::from("-R")];
        let forwards: Vec<String> = bootstrap_data.forwards.iter()
                    .map(|f| format!("{}:localhost:{}", f.remote, f.local)).collect();

        let full_args: Vec<&str> = initial_args.iter()
            .chain(forwards.iter())
            .chain(connection_sharing_args.iter())
            .chain(parsed_ssh_args.ssh_args.iter())
            .chain(std::iter::once(&parsed_ssh_args.server_args[0]))
            .chain(std::iter::once(&bootstrap_data.script))
            .map(|s| s.as_str())
            .collect();

            // debug!("ssh {:?}", full_args[..full_args.lenv)-1]);

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
