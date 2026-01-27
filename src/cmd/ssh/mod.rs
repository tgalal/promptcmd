use std::{path::PathBuf, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use clap::{Parser};
use anyhow::{Context, Result};
use tokio::process::Command;
use crate::{cmd::ssh::shell::{Channel, Shell}, config::appconfig::{AppConfig, ChannelOptions, Remote, ShellOptions}, executor::{Executor, MultiplexedSession}};
pub mod controlpath;
pub mod utils;
use rand::Rng;

mod shell;
mod lchannel;
mod bootstrap;

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
        appconfig: &AppConfig
    )-> Result<()> {

        let ssh_args: Vec<String> = self.ssh_args.clone();

        // println!("{:#?}", &session_info);
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

        let bootstrap_data = bootstrap::setup(executor, &prompts, shell, channel);

        // println!("{}", &bootstrap_data.script);

        tokio::spawn(async move {
            bootstrap_data.lchannel.run().await.context("Channel Error")?;
            Ok::<(), anyhow::Error>(())
        });


        // println!("Fwd: {:#?}", &bootstrap_data.forwards);

        let connection_sharing_args = vec![
            "-o".to_string(),
            "ControlMaster=yes".to_string(),
            "-o".to_string(),
            format!("ControlPath={}", &controlpath),
            "-o".to_string(),
            "ControlPersist=no".to_string(),
        ];


        let ssh_cmd_handle = tokio::spawn(async move {
            Command::new("ssh")
                .arg("-t")
                .arg("-R")
                .args(&bootstrap_data.forwards)
                //.arg(&forwarding_arg)
                // .arg("9999:localhost:9999")
                .args(connection_sharing_args)
                .args(ssh_args)
                .arg(&bootstrap_data.script)
                .spawn().context("Error spawning ssh")?
                .wait().await.context("Error in ssh proc")?;
            Ok::<(), anyhow::Error>(())
        });

        ssh_cmd_handle.await??;

        Ok(())
    }
}
