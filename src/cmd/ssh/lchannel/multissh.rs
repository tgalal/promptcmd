use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Child;
use tokio::process::ChildStdin;
use tokio::process::ChildStdout;
use tokio::process::Command;

use crate::cmd;
use crate::cmd::ssh::lchannel::stream_common::HandleResult;
use crate::cmd::ssh::lchannel::LChannel;
use crate::cmd::ssh::utils::WaitForMasterResult;
use crate::executor::Executor;
use crate::executor::MultiplexedSession;
use log::debug;
use log::error;
use super::stream_common;
use super::ChannelError;
use async_trait::async_trait;


pub struct MultiSshChannel {
    pub executor: Arc<Executor>,
    pub session: MultiplexedSession,
    pub rendezvous_path: String,
}

struct SshFifoListener {
    child: Child,
    workdir: String,
    session: MultiplexedSession,
    reader: BufReader<ChildStdout>,
}

struct FifoStream {
    child: Child,
    stdout: ChildStdout,
    stdin: ChildStdin
}

impl SshFifoListener {

    pub async fn accept(&mut self) -> Result<FifoStream, ChannelError> {
        let mut buf = String::new();
        debug!("Waiting for readline");
        self.reader.read_line(&mut buf).await?;
        debug!("Got readline: {buf}");

        if let Some((_, identifier)) = buf.split_once("CONN ") {
            let identifier = identifier.trim();

            let send_path = format!("{}/{identifier}_send", self.workdir);
            let recv_path = format!("{}/{identifier}_recv", self.workdir);

            let remote_command = format!(r#"
cat {send_path};
cat >> {recv_path};
"#);

            debug!("Remote cmd: {remote_command}");
            let mut child =  {
                Command::new("ssh")
                .arg("-S")
                .arg(&self.session.controlpath)
                .arg("-p")
                .arg(self.session.port.to_string())
                .arg("-T")
                .arg(&self.session.destination.hostname)
                .arg(&remote_command)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                //.kill_on_drop(true)
                .spawn()?
            };

            debug!("Created handler, PID: {}", child.id().unwrap());

            let stdin = child.stdin.take().expect("Failed to open stdin");
            let stdout = child.stdout.take().expect("Failed to open stdout");

            return Ok(FifoStream {
                child,
                stdin,
                stdout
            })
        }

        if buf.is_empty() {
            Err(ChannelError::EOF)
        } else {
            Err(ChannelError::Other(format!("Unexpected message: {buf}")))
        }
    }

    pub async fn close(mut self) {
        debug!("Killing child with {}", self.child.id().unwrap());
        self.child.kill().await.unwrap();
        drop(self.child);
    }

    pub async fn bind(session: &MultiplexedSession, rendezvouz: &str) -> Result<Self, ChannelError> {
        let workdir = PathBuf::from(rendezvouz).parent().map(|p| p.to_string_lossy().to_string()).unwrap();
        debug!("Remote work dir is: {workdir}");

        let remote_command = format!(r#"sh -c "
mkdir -m 700 -p {workdir};
[ -p {rendezvous_path} ] || mkfifo -m 600 {rendezvous_path};
while true; do
    line=\$(cat {rendezvous_path})
    case \"\$line\" in
        __exit__) break ;;
        *)    printf '%s\n' \"\$line\" ;;
    esac
done
"
"#,
            rendezvous_path=rendezvouz);

        debug!("Remote cmd: {remote_command}");
        let mut child =  {
            Command::new("ssh")
            .arg("-S")
            .arg(&session.controlpath)
            .arg("-p")
            .arg(session.port.to_string())
            .arg("-T")
            .arg(&session.destination.hostname)
            .arg(&remote_command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()?
        };
        debug!("Created session, PID: {}", child.id().unwrap());

        // let stdin = child.stdin.take().expect("Failed to open stdin");
        let stdout = child.stdout.take().expect("Failed to open stdout");
        let reader = BufReader::new(stdout);

        Ok(
            Self {
                child,
                workdir: workdir.clone(),
                session: session.clone(),
                reader
            }
        )
    }
}

#[async_trait]
impl LChannel for MultiSshChannel {
    async fn run(&self) -> Result<(), ChannelError> {

        // println!("Waiting 30 seconds for master connection to succeed");
        // println!("{:#?}", self.session);
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

        debug!("Binding Listener");
        let mut listener = SshFifoListener::bind(&self.session, &self.rendezvous_path).await?;

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let shutdown_tx = std::sync::Arc::new(tokio::sync::Mutex::new(Some(shutdown_tx)));

        debug!("Starting accept loop");
        loop {
            tokio::select! {
                 _ = &mut shutdown_rx => {
                    debug!("Shutdown signal received, closing listener");
                    listener.close().await;
                    break;
                }
                result = listener.accept() => {
                    debug!("Waiting for fifo connection");
                    let mut stream = result?;
                    debug!("Handling fifo connection");

                    let executor = self.executor.clone();
                    let (reader, writer) = (stream.stdout, stream.stdin);

                    let shutdown_tx = shutdown_tx.clone();
                    tokio::spawn(async move {
                        let res = stream_common::handle_stream(executor, reader, writer).await;

                        match res {
                            Ok(HandleResult::Continue) => {},
                            Ok(HandleResult::Exit) => {
                                if let Some(tx) = shutdown_tx.lock().await.take() {
                                    let _ = tx.send(());
                                }
                            },
                            Err(err) => {
                                error!("{err}");
                            }
                        }
                        stream.child.kill().await
                    });
                }
            }
        }
        Ok(())
    }
}
