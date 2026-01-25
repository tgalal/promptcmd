use std::sync::Arc;

use clap::{Parser};
use anyhow::{Context, Result};
use tokio::process::Command;
use crate::executor::{Executor, MultiplexedSession};
mod tcp_channel;
mod bootstrap;
pub mod controlpath;
pub mod utils;


#[derive(Parser)]
pub struct SshCmd {
    #[arg(trailing_var_arg = true, allow_hyphen_values=true)]
    pub ssh_args: Vec<String>,
}

impl SshCmd {
    pub async fn exec(&self,
        executor: Arc<Executor>,
        prompts: Vec<String>,
        session_info: MultiplexedSession
    )-> Result<()> {

        let ssh_args: Vec<String> = self.ssh_args.clone();
        let bootstrap_script = bootstrap::generate_ssh_bootstrap_command(&prompts, 9999);

        tokio::spawn(async {
            let dispatcher = tcp_channel::TcpChannelDispatcher {
                executor,
            };
            dispatcher.new_channel().await.context("Error creating channel")?;
            Ok::<(), anyhow::Error>(())
        });


        let controlpath = session_info.controlpath;

        let connection_sharing_args = vec![
            "-o".to_string(),
            "ControlMaster=yes".to_string(),
            "-o".to_string(),
            format!("ControlPath={}", &controlpath),
            "-o".to_string(),
            "ControlPersist=no".to_string(),
        ];

        let ssh_cmd_handle = tokio::spawn(async {
            Command::new("ssh")
                .arg("-t")
                .arg("-R")
                .arg("9999:localhost:9999")
                .args(connection_sharing_args)
                .args(ssh_args)
                .arg(bootstrap_script)
                .spawn().context("Error spawning ssh")?
                .wait().await.context("Error in ssh proc")?;
            Ok::<(), anyhow::Error>(())
        });

        ssh_cmd_handle.await??;

        Ok(())
    }
}
