use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;

use crate::cmd;
use crate::cmd::ssh::lchannel::stream_common::HandleResult;
use crate::cmd::ssh::lchannel::LChannel;
use crate::cmd::ssh::utils::WaitForMasterResult;
use crate::executor::Executor;
use crate::executor::MultiplexedSession;
use log::debug;
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

        let conn_state = cmd::ssh::utils::async_wait_for_master_ready(
            &self.session.controlpath,
            &self.session.destination.hostname,
            self.session.port,
            Duration::from_secs(120),
            None
        ).await.map_err(|e|
                ChannelError::Other(e.to_string())
        )?;

        match conn_state {
            WaitForMasterResult::Established => {
                debug!("Master connection succeeded, proceeding.");
            },
            WaitForMasterResult::Aborted => {
                debug!("Aborted Connection");
                return Ok(())
            },
            WaitForMasterResult::Timeout => {
                debug!("Timeout waiting for master connection");
                return Err(ChannelError::TimeoutError)
            }
        }

        let workdir = PathBuf::from(&self.send_path).parent().map(|p| p.to_string_lossy().to_string()).unwrap();
        debug!("Remote work dir is: {workdir}");

        let session_info = &self.session;

        let remote_command = format!(r#"
sh -c "mkdir -p {workdir};
[ -p {workdir}/send ] || mkfifo -m 600 {workdir}/send;
[ -p {workdir}/recv ] || mkfifo -m 600 {workdir}/recv;
cat {send_path}; cat >> {recv_path}
"
"#,
            send_path = self.send_path, recv_path = self.recv_path);

        loop {
            debug!("Remote cmd: {remote_command}");
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

        Ok(())
    }
}
