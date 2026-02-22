use std::sync::Arc;

use clap::Arg;
use clap::{Command as ClapCommand};
use log::debug;
use log::warn;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::dotprompt::renderers::argmatches::DotPromptArgMatches;
use crate::executor::{ExecutionOutput, PromptInputs};
use crate::{cmd::run, executor::Executor};
use super::ChannelError;

pub enum HandleResult {
    Continue,
    Exit
}

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
    let command_line = String::from_utf8(buffer)?.trim().to_string();

    Ok(command_line)
}

async fn authenticate<S: AsyncReadExt + Unpin>(stream: &mut S, session_pwd: &str) -> Result<bool, ChannelError> {
    let mut buffer = Vec::new();
    loop {
        let byte = stream
            .read_u8()
            .await?;

        // 1= \n
        if byte == 10 {
            break;
        }

        buffer.push(byte);
    }
    let password = String::from_utf8(buffer)?.trim().to_string();

    debug!("Expected Pwd: {session_pwd}, Got: {password}, Auth: {}", password == session_pwd);
    Ok(password == session_pwd)
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

pub async fn handle_stream<READ, WRITE>(executor: Arc<Executor>, mut in_stream: READ,
    mut out_stream: WRITE, session_pwd: &str) -> Result<HandleResult, ChannelError>
    where
    READ: AsyncReadExt + Unpin,
    WRITE: AsyncWriteExt + Unpin

{
    debug!("Handling incoming connection");

    if !authenticate(&mut in_stream, session_pwd).await? {
        return Err(ChannelError::AuthError);
    }

    let command_full = read_command(&mut in_stream).await?;
    debug!("Got Command: {command_full}");
    if command_full == "__exit__"  {
        return Ok(HandleResult::Exit)
    }
    let stdin = read_stdin(&mut in_stream).await?;

    let (promptname, command_args_string) = command_full.split_once(" ")
        .unwrap_or((command_full.as_str(), ""));

    debug!("Prompt name: {promptname}");
    debug!("Prompt args: {command_args_string}");

    let dotprompt = executor.load_dotprompt(promptname).unwrap();

    let mut command: ClapCommand = ClapCommand::new(promptname.to_string());
    // command = command.disable_help_flag(true);
    //
    command = command.arg(Arg::new("dry")
            .long("dry")
            .help("Dry run")
            .action(clap::ArgAction::SetTrue)
            .required(false)
        )
        .arg(Arg::new("render")
            .long("render")
            .short('r')
            .help("Render only mode")
            .action(clap::ArgAction::SetTrue)
            .required(false)
        );
    command = command.next_help_heading("Prompt inputs");
    command = run::generate_arguments_from_dotprompt(command, &dotprompt).unwrap();
    command = command.next_help_heading("Optional Configuration Overrides")
        .arg(Arg::new("model")
            .long("config-model")
            .short('m')
        );
    let args = shlex::split(command_full.as_str()).expect("Failed to parse command line");
    let matches = command.try_get_matches_from(args);


    match matches {
        Ok(matches) => {

            let dry = *matches.get_one::<bool>("dry").unwrap_or(&false);
            let render = *matches.get_one::<bool>("render").unwrap_or(&false);
            let requested_model = matches.get_one::<String>("model").map(|s| s.to_string());

            let argmatches = DotPromptArgMatches {
                matches,
                dotprompt: &dotprompt
            };

            let inputs: PromptInputs = argmatches.try_into().unwrap();

            let result = executor.clone().execute_dotprompt(&dotprompt,
                None, requested_model,
                inputs, Some(stdin), dry, render).await.unwrap();


            match result {
                // _ => {panic!("Streaming Unsupported")}
                ExecutionOutput::StreamingOutput(mut tokstream) => {
                    let mut ends_with_newline = false;

                    while let Some(res) = tokstream.sync_next().await {
                        let data_str = res.unwrap();
                        let data = data_str.as_bytes();

                        ends_with_newline = data_str.ends_with("\n");
                        if let Err(err) = out_stream.write_all(data).await {
                            warn!("{err}");
                            break;
                        }
                        if let Err(err) = out_stream.flush().await {
                            warn!("{err}");
                            break;
                        }
                    }
                    if !ends_with_newline  && let Err(err) = out_stream.write_all("\n".as_bytes()).await {
                        warn!("{err}");
                    }
                }
                ExecutionOutput::StructuredStreamingOutput(mut tokstream) => {
                    let mut ends_with_newline = false;

                    while let Some(res) = tokstream.sync_next().await {
                        let data_str = res.unwrap();
                        let data = data_str.as_bytes();

                        ends_with_newline = data_str.ends_with("\n");
                        if let Err(err) = out_stream.write_all(data).await {
                            warn!("{err}");
                            break;
                        }
                        if let Err(err) = out_stream.flush().await {
                            warn!("{err}");
                            break;
                        }
                    }
                    if !ends_with_newline  && let Err(err) = out_stream.write_all("\n".as_bytes()).await {
                        warn!("{err}");
                    }
                }
                ExecutionOutput::ImmediateOutput(output) => {
                    out_stream.write_all(output.as_bytes()).await.unwrap();
                    if !output.ends_with("\n") {
                        out_stream.write_all("\n".as_bytes()).await.unwrap();
                    }
                    out_stream.flush().await.unwrap();
                }
                ExecutionOutput::DryRun => {
                    // println!("[dry run, no llm response]");
                    out_stream.write_all(b"[dry run, no llm response]").await.unwrap();
                    out_stream.flush().await.unwrap();
                }
                ExecutionOutput::Cached(output) => {
                    out_stream.write_all(output.as_bytes()).await.unwrap();
                    if !output.ends_with("\n") {
                        out_stream.write_all("\n".as_bytes()).await.unwrap();
                    }
                    out_stream.flush().await.unwrap();
                }
                ExecutionOutput::RenderOnly(output) => {
                    out_stream.write_all(output.as_bytes()).await.unwrap();
                    out_stream.flush().await.unwrap();
                }
            };
        }
        Err(err) => {
            out_stream.write_all(err.to_string().as_bytes()).await.unwrap();
        }
    }
    Ok(HandleResult::Continue)

}
