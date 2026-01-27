use std::{path::PathBuf, sync::Arc};

use clap::{Parser};
use anyhow::{Context, Result};
use tokio::process::Command;
use crate::{executor::{Executor, MultiplexedSession}};
pub mod controlpath;
pub mod utils;

mod shell;
mod lchannel;
mod rchannel;


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
        session_info: MultiplexedSession
    )-> Result<()> {

        let ssh_args: Vec<String> = self.ssh_args.clone();
        let functions = rchannel::bashtcp::create_functions(&prompts, 9999);
        // let functions = rchannel::socat::create_functions(&prompts, "/tmp/pcmd.sock");
        let bootstrap_script = shell::auto::setup(REMOTE_WORKDIR, &prompts, &functions);


        let controlpath = session_info.controlpath;
        let usock_path = PathBuf::from(&controlpath)
            .parent()
            .context("Error getting control path's parent dir")?
            .join("pcmd.sock");

        let usock_path_str = usock_path.to_string_lossy().to_string();

        tokio::spawn(async {
            let channel = lchannel::tcp::TcpChannel {
                executor ,
                port: 9999
            };
            // let channel = lchannel::usock::USocketChannel {
            //     executor,
            //     path: usock_path
            // };
            channel.run().await.context("Channel Error")?;
            Ok::<(), anyhow::Error>(())
        });

        // let forwarding_arg = format!("/tmp/pcmd.sock:{}", usock_path_str);
        let forwarding_arg = format!("{rport}:localhost:{lport}", rport=9999, lport=9999);
        println!("Fwd: {forwarding_arg}");

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
                .arg(&forwarding_arg)
                // .arg("9999:localhost:9999")
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
