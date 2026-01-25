use std::{io::Read, process::Command};
use log::debug;

use handlebars::*;
use tokio::io::AsyncReadExt;

use openssh::{KnownHosts, Session, Stdio};

use crate::executor::{MultiplexedSession, RemoteExecContext};
pub struct RemoteExecHelper {
    pub context: RemoteExecContext
}

impl RemoteExecHelper {
    async fn handle_multiplexed_session(&self, session_info: &MultiplexedSession, cmd: String, args: Vec<String>,
        out: &mut dyn Output,
    ) -> HelperResult {

        let (mut reader, writer) = std::io::pipe()?;

        debug!("{:#?}", &args);

        let child =  {
            Command::new("ssh")
            .arg("-S")
            .arg(&session_info.controlpath)
            .arg(&session_info.destination.hostname)
            .arg(&cmd)
            .args(&args)
            .stdout(writer.try_clone()?)
            .stderr(writer)
            .output()?
        };

        let mut output = String::new();
        reader.read_to_string(&mut output)?;

        if child.status.success() {
            out.write(&output)?;
            Ok(())
        } else {
            let error_message = format!("Error executing command: {}, output was: {}", &cmd, &output);
            Err(RenderError::from(RenderErrorReason::Other(error_message)))
        }
    }

    async fn handle_destination(&self, destination: &str, cmd: String, args: Vec<String>,
        out: &mut dyn Output,
    ) -> HelperResult {
        // Connect to remote host
        let session = Session::connect(destination, KnownHosts::Strict)
            .await
            .map_err(|err| RenderError::from(RenderErrorReason::Other(err.to_string())))?;

        println!("Socket: {}", session.control_socket().to_string_lossy());
        let mut child = session.command(&cmd)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        .await
        .map_err(|err| RenderError::from(RenderErrorReason::Other(err.to_string())))?;

        // Read both streams concurrently
        let stdout = child.stdout().take().unwrap();
        let stderr = child.stderr().take().unwrap();
        let mut stdout_data = Vec::new();
        let mut stderr_data = Vec::new();
        let (stdout_result, stderr_result) = tokio::join!(
            async {
                let mut reader = stdout;
                reader.read_to_end(&mut stdout_data).await
            },
            async {
                let mut reader = stderr;
                reader.read_to_end(&mut stderr_data).await
            }
        );
        stdout_result?;
        stderr_result?;

        let exit_status = child.wait()
            .await
            .map_err(|err| RenderError::from(RenderErrorReason::Other(err.to_string())))?;

        session.close()
            .await
            .map_err(|err| RenderError::from(RenderErrorReason::Other(err.to_string())))?;
        // Combine them
        let mut combined = stdout_data;
        combined.extend_from_slice(&stderr_data);
        let output = String::from_utf8_lossy(&combined).to_string();

        if exit_status.success() {
            out.write(&output)?;
            Ok(())
        } else {
            let error_message = format!("Error executing command: {}, output was: {}", &cmd, &output);
            Err(RenderError::from(RenderErrorReason::Other(error_message)))
        }
    }

    async fn async_call<'reg: 'rc, 'rc>( &self,
            h: &Helper<'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {
        let params = h.params();
        let cmd = params.first().ok_or(
            RenderError::from(RenderErrorReason::Other("exec binary not specified".to_string()))
        )?.render();

        let args: Vec<String> = params.iter().skip(1).map(|item| {
            if item.is_value_missing() {
                Err(RenderError::from(RenderErrorReason::Other(
                    format!("Undefined variable: {}", item.relative_path().unwrap()))))
            } else {
                Ok(item.render())
            }
        }).collect::<Result<Vec<_>, _>>()?;

        match &self.context {
            RemoteExecContext::Destination(destination) => self.handle_destination(destination.as_str(), cmd, args, out).await,
            RemoteExecContext::MultiplexedSession(session_info) => self.handle_multiplexed_session(
                session_info, cmd, args, out).await
        }

    }
}

impl HelperDef for RemoteExecHelper {
    fn call<'reg: 'rc, 'rc>( &self,
            h: &Helper<'rc>,
            _: &'reg Handlebars<'reg>,
            _: &'rc Context,
            _: &mut RenderContext<'reg, 'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.async_call(h, out).await
            })
        })

    }
}
