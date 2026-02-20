mod exec;
mod prompt;
mod concat;
mod stdin;
mod ask;
mod cat;
mod tail;
mod head;
mod remote_exec;
mod remote_cat;
mod remote_tail;
mod remote_head;

pub use exec::ExecHelper;
pub use prompt::PromptHelper;
pub use concat::ConcatHelper;
pub use stdin::StdinHelper;
pub use ask::AskHelper;
pub use cat::CatHelper;
pub use tail::TailHelper;
pub use head::HeadHelper;
pub use remote_exec::RemoteExecHelper;
pub use remote_cat::RemoteCatHelper;
pub use remote_tail::RemoteTailHelper;
pub use remote_head::RemoteHeadHelper;


use openssh::{KnownHosts, Session, Stdio};
use crate::{executor::MultiplexedSession};
use log::debug;
use handlebars::{Output, HelperResult, RenderError, RenderErrorReason};
use std::{io::Read, process::Command};
use tokio::io::AsyncReadExt;

async fn handle_destination(destination: &str, cmd: &str, args: &[String],
    out: &mut dyn Output,
) -> HelperResult {
    // Connect to remote host
    let session = Session::connect(destination, KnownHosts::Strict)
        .await
        .map_err(|err| RenderError::from(RenderErrorReason::Other(err.to_string())))?;

    let mut child = session.command(cmd)
        .args(args)
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

async fn handle_multiplexed_session(session_info: &MultiplexedSession, cmd: &str, args: &[String],
    out: &mut dyn Output,
) -> HelperResult {

    let (mut reader, writer) = std::io::pipe()?;

    debug!("{:#?}", &args);

    let child =  {
        Command::new("ssh")
        .arg("-S")
        .arg(&session_info.controlpath)
        .arg("-p")
        .arg(session_info.port.to_string())
        .arg(&session_info.destination.hostname)
        .arg(cmd)
        .args(args)
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


fn handle_local_cmd(cmd: &str, args: &[String], out: &mut dyn Output,
) -> HelperResult {

        let (mut reader, writer) = std::io::pipe()?;

        let child =  {
            Command::new(cmd)
            .args(args)
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
