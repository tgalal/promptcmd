use std::sync::Arc;

use clap::{Command as ClapCommand};
use log::debug;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::dotprompt::renderers::argmatches::DotPromptArgMatches;
use crate::executor::{ExecutionOutput, PromptInputs};
use crate::{cmd::run, executor::Executor};
use super::ChannelError;

async fn read_command<S: AsyncReadExt + Unpin>(stream: &mut S) -> Result<String, ChannelError> {
    let mut buffer = Vec::new();
    loop {
        let byte = stream
            .read_u8()
            .await?;

        if byte == 0 {
            break;
        }

        buffer.push(byte);
    }
    let command_line = String::from_utf8(buffer)?;
    debug!("Got command: {command_line}");
    Ok(command_line)
}

async fn read_stdin<S: AsyncReadExt + Unpin>(stream: &mut S) -> Result<String, ChannelError> {
    let mut buffer = Vec::new();
    loop {
        let byte = stream
            .read_u8()
            .await?;

        if byte == 0 {
            break;
        }

        buffer.push(byte);
    }
    let stdin = String::from_utf8(buffer)?;
    debug!("Got stdin: {stdin}");
    Ok(stdin)
}

pub async fn handle_stream<S: AsyncReadExt + AsyncWriteExt + Unpin>(executor: Arc<Executor>, mut stream: S) -> Result<(), ChannelError> {
    debug!("Handling incoming connection");

    let command_full = read_command(&mut stream).await?;
    let stdin = read_stdin(&mut stream).await?;

    let (promptname, command_args_string) = command_full.split_once(" ")
        .unwrap_or((command_full.as_str(), ""));

    debug!("Prompt name: {promptname}");
    debug!("Prompt args: {command_args_string}");

    let dotprompt = executor.load_dotprompt(promptname).unwrap();

    let mut command: ClapCommand = ClapCommand::new(promptname.to_string());
    // command = command.disable_help_flag(true);
    command = command.next_help_heading("Prompt inputs");
    command = run::generate_arguments_from_dotprompt(command, &dotprompt).unwrap();
    let args = shlex::split(command_full.as_str()).expect("Failed to parse command line");
    let matches = command.try_get_matches_from(args);

    match matches {
        Ok(matches) => {
            let argmatches = DotPromptArgMatches {
                matches,
                dotprompt: &dotprompt
            };

            let inputs: PromptInputs = argmatches.try_into().unwrap();

            let result = executor.clone().execute_dotprompt(&dotprompt,
                None, None,
                inputs, Some(stdin), false, false).await.unwrap();


            match result {
                // _ => {panic!("Streaming Unsupported")}
                ExecutionOutput::StreamingOutput(mut tokstream) => {
                    let mut ends_with_newline = false;

                    while let Some(res) = tokstream.sync_next().await {
                        let data_str = res.unwrap();
                        let data = data_str.as_bytes();

                        ends_with_newline = data_str.ends_with("\n");
                        stream.write_all(data).await.unwrap();
                        stream.flush().await.unwrap();
                    }
                    if !ends_with_newline {
                        stream.write_all("\n".as_bytes()).await.unwrap();
                    }
                }
                ExecutionOutput::StructuredStreamingOutput(mut tokstream) => {
                    let mut ends_with_newline = false;

                    while let Some(res) = tokstream.sync_next().await {
                        let data_str = res.unwrap();
                        let data = data_str.as_bytes();

                        ends_with_newline = data_str.ends_with("\n");
                        stream.write_all(data).await.unwrap();
                        stream.flush().await.unwrap();
                    }
                    if !ends_with_newline {
                        stream.write_all("\n".as_bytes()).await.unwrap();
                    }
                }
                ExecutionOutput::ImmediateOutput(output) => {
                    debug!("Going to write {output}" );
                    stream.write_all(output.as_bytes()).await.unwrap();
                    if !output.ends_with("\n") {
                        stream.write_all("\n".as_bytes()).await.unwrap();
                    }
                    stream.flush().await.unwrap();
                    debug!("Finishe Writing");
                }
                ExecutionOutput::DryRun => {
                    // println!("[dry run, no llm response]");
                    stream.write_all(b"[dry run, no llm response]").await.unwrap();
                    stream.flush().await.unwrap();
                }
                ExecutionOutput::Cached(output) => {
                    stream.write_all(output.as_bytes()).await.unwrap();
                    if !output.ends_with("\n") {
                        stream.write_all("\n".as_bytes()).await.unwrap();
                    }
                    stream.flush().await.unwrap();
                }
                ExecutionOutput::RenderOnly(output) => {
                    stream.write_all(output.as_bytes()).await.unwrap();
                    stream.flush().await.unwrap();
                }
            };
        }
        Err(err) => {
            stream.write_all(err.to_string().as_bytes()).await.unwrap();
        }
    }
    Ok(())

}
