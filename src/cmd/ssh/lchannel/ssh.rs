use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::{sleep};

use crate::cmd;
use crate::cmd::ssh::lchannel::stream_common::HandleResult;
use crate::cmd::ssh::lchannel::LChannel;
use crate::executor::Executor;
use crate::executor::MultiplexedSession;
use super::stream_common;
use super::ChannelError;
use async_trait::async_trait;


pub struct SshChannel {
    pub executor: Arc<Executor>,
    pub session: MultiplexedSession,
    pub send_path: String,
    pub recv_path: String
}

#[async_trait]
impl LChannel for SshChannel {
    async fn run(&self) -> Result<(), ChannelError> {

        // println!("Waiting 30 seconds for master connection to succeed");
        // println!("{:#?}", self.session);
        cmd::ssh::utils::async_wait_for_master_ready(
            &self.session.controlpath,
            &self.session.destination.hostname,
            self.session.port,
            Duration::from_secs(30)
        ).await.map_err(|_|
                ChannelError::TimeoutError
        )?;
        // println!("Master connection succeeded, proceeding.");

        // wait additional 500ms for bootstrap script to execute
        sleep(Duration::from_millis(2000)).await;

        let session_info = &self.session;
        //let remote_command = format!("rm {recv_path} {send_path} 2> /dev/null; mkfifo {recv_path}; mkfifo {send_path} && cat {send_path}; cat >> {recv_path}",
        //    send_path = self.send_path, recv_path = self.recv_path);
        let remote_command = format!("cat {send_path}; cat >> {recv_path}",
            send_path = self.send_path, recv_path = self.recv_path);

        // println!("{remote_command}");

        loop {
            let mut child =  {
                Command::new("ssh")
                .arg("-S")
                .arg(&session_info.controlpath)
                .arg("-p")
                .arg(session_info.port.to_string())
                .arg("-T")
                .arg(&session_info.destination.hostname)
                .arg(&remote_command)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .kill_on_drop(true)
                .spawn()?
            };

            // Get handles to stdin and stdout
            let stdin = child.stdin.take().expect("Failed to open stdin");
            let stdout = child.stdout.take().expect("Failed to open stdout");

            let handle_result = stream_common::handle_stream(
                self.executor.clone(),
                stdout,
                stdin
            ).await?;

            match handle_result {
                HandleResult::Exit => {
                    break;
                }
                HandleResult::Continue => {
                    continue;
                }
            }
        }

        // println!("Terminating the process");

        Ok(())

        //let stdout_task = tokio::spawn(async move {
        //    let reader = BufReader::new(stdout);
        //    let mut lines = reader.lines();

        //    while let Ok(Some(line)) = lines.next_line().await {
        //        println!("OUTPUT: {}", line);
        //    }
        //});


        // loop {
        //     let (tcpstream, _) =listener.accept().await?;
        //     let executor = self.executor.clone();

        //     tokio::spawn(async {
        //         stream_common::handle_stream(executor, tcpstream).await
        //     });
        // }
    }
}
