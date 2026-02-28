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

use crate::{executor};

use handlebars::{Output, HelperResult, RenderError, RenderErrorReason};
use std::{io::Read, process::Command};

#[cfg(not(target_os="windows"))]
async fn handle_multiplexed_session(session_info: &executor::MultiplexedSession, cmd: &str, args: &[String],
    out: &mut dyn Output,
) -> HelperResult {

    let (mut reader, writer) = std::io::pipe()?;

    log::debug!("{:#?}", &args);

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
