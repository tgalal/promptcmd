use std::{string::FromUtf8Error, sync::Arc};

use clap::{Command as ClapCommand};
use log::debug;
use tokio::net::{TcpListener, TcpStream};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::dotprompt::renderers::argmatches::DotPromptArgMatches;
use crate::executor::{ExecutionOutput, PromptInputs};
use crate::{cmd::run, executor::Executor};

#[derive(Error, Debug)]
pub enum TcpChannelError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Encoding Error: {0}")]
    EncodingError(#[from] FromUtf8Error),
}

pub struct TcpChannelDispatcher {
    pub executor: Arc<Executor>,
}

impl TcpChannelDispatcher {
    pub async fn new_channel(&self) -> Result<(), TcpChannelError> {
        let channel = TcpChannel::new(
            9999, self.executor.clone(),
        ).await?;
        channel.run().await?;
        Ok(())
    }
}

struct TcpChannel {
    listener: TcpListener,
    pub executor: Arc<Executor>,
}

impl TcpChannel {

    pub async fn new(
        port: u32,
        executor: Arc<Executor>,
    ) -> Result<Self, TcpChannelError> {
        let addr = format!("127.0.0.1:{port}");
        let listener = TcpListener::bind(&addr)
            .await?;


        Ok(Self {
            listener,
            executor,
        })
    }


    // Run the server, accepting and handling connections
    pub async fn run(&self) -> Result<(), TcpChannelError> {
        loop {
            let (tcpstream, _) = self
                .listener
                .accept()
                .await?;

            let executor = self.executor.clone();

            tokio::spawn(async {
                handle_connection(executor,  tcpstream).await
            });
        }
    }
}
async fn handle_connection(executor: Arc<Executor>, mut tcpstream: TcpStream) -> Result<(), TcpChannelError> {
    debug!("Handling incoming connection");
    let mut buffer = Vec::new();
    // read until first unescaped new line
    loop {
        let byte = tcpstream
            .read_u8()
            .await?;

        // 10 is new line char
        // TODO: check if escaped
        if byte == 10 {
            break;
        }

        buffer.push(byte);
    }
    let command_full = String::from_utf8(buffer)?;
    let (promptname, command_args_string) = command_full.split_once(" ")
        .unwrap_or((command_full.as_str(), ""));
    // let command_split = command_full.split_once(" ").unwrap_or(command_full);
    // let promptname = command_split.0.to_string();
    // let command_args_string = command_split.1.to_string();


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
                inputs, false, false).await.unwrap();


            match result {
                // _ => {panic!("Streaming Unsupported")}
                ExecutionOutput::StreamingOutput(mut stream) => {
                    let mut ends_with_newline = false;

                    while let Some(res) = stream.sync_next().await {
                        let data_str = res.unwrap();
                        let data = data_str.as_bytes();

                        ends_with_newline = data_str.ends_with("\n");
                        tcpstream.write_all(data).await.unwrap();
                        tcpstream.flush().await.unwrap();
                    }
                    if !ends_with_newline {
                        tcpstream.write_all("\n".as_bytes()).await.unwrap();
                    }
                }
                ExecutionOutput::StructuredStreamingOutput(mut stream) => {
                    let mut ends_with_newline = false;

                    while let Some(res) = stream.sync_next().await {
                        let data_str = res.unwrap();
                        let data = data_str.as_bytes();

                        ends_with_newline = data_str.ends_with("\n");
                        tcpstream.write_all(data).await.unwrap();
                        tcpstream.flush().await.unwrap();
                    }
                    if !ends_with_newline {
                        tcpstream.write_all("\n".as_bytes()).await.unwrap();
                    }
                }
                ExecutionOutput::ImmediateOutput(output) => {
                    tcpstream.write_all(output.as_bytes()).await.unwrap();
                    if !output.ends_with("\n") {
                        tcpstream.write_all("\n".as_bytes()).await.unwrap();
                    }
                    tcpstream.flush().await.unwrap();
                }
                ExecutionOutput::DryRun => {
                    // println!("[dry run, no llm response]");
                    tcpstream.write_all(b"[dry run, no llm response]").await.unwrap();
                    tcpstream.flush().await.unwrap();
                }
                ExecutionOutput::Cached(output) => {
                    tcpstream.write_all(output.as_bytes()).await.unwrap();
                    if !output.ends_with("\n") {
                        tcpstream.write_all("\n".as_bytes()).await.unwrap();
                    }
                    tcpstream.flush().await.unwrap();
                }
                ExecutionOutput::RenderOnly(output) => {
                    tcpstream.write_all(output.as_bytes()).await.unwrap();
                    tcpstream.flush().await.unwrap();
                }
            };
        }
        Err(err) => {
            tcpstream.write_all(err.to_string().as_bytes()).await.unwrap();
        }
    }
    Ok(())

}
