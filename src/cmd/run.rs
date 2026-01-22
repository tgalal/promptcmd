use clap::{Parser};
use clap::{Arg, Command};
use std::io::{self, Write};
use std::sync::Arc;
use std::convert::TryFrom;
use anyhow::{Context, Result};
use thiserror::Error;
use crate::dotprompt::renderers::argmatches::DotPromptArgMatches;
use crate::executor::{ExecutionOutput, Executor, PromptInputs};
use crate::dotprompt::{ DotPrompt};

#[derive(Parser)]
pub struct RunCmd {
    #[arg()]
    pub promptname: String,

    #[arg(long, short, help="Dry run" )]
    pub dry: bool,

    #[arg(long, short, help="Render only" )]
    pub render: bool,

    #[arg(trailing_var_arg = true)]
    pub prompt_args: Vec<String>,
}

pub fn generate_arguments_from_dotprompt(mut command: Command, dotprompt: &DotPrompt) -> Result<Command> {
    let args: Vec<Arg> = Vec::try_from(dotprompt).context(
        "Could not generate arguments"
    )?;

    for arg in args {
       command = command.arg(arg);
    }
    Ok(command)
}

#[derive(Error, Debug)]
pub enum RunCmdError {
    #[error("'{0}' is required but not configured")]
    RequiredConfiguration(&'static str)
}

impl RunCmd {
    pub fn exec(&self,
        executor: Arc<Executor>,
        ) -> Result<()> {

        let dotprompt: DotPrompt = executor.load_dotprompt(&self.promptname)?;

        let mut command: Command = Command::new(self.promptname.to_string());

        command = generate_arguments_from_dotprompt(command, &dotprompt)?;

        let params = [vec!["--".to_string()], self.prompt_args.clone()].concat();
        let matches = command.get_matches_from(params);

        let argmatches = DotPromptArgMatches {
            matches,
            dotprompt: &dotprompt
        };

        let inputs: PromptInputs = argmatches.try_into()?;

        let result = executor.execute_dotprompt(&dotprompt, None,
            None, inputs, self.dry, self.render)?;

        match result{
            ExecutionOutput::StreamingOutput(mut stream) => {
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                let mut ends_with_newline = false;

                while let Some(res) = stream.sync_next() {
                    let data_str = res?;
                    let data = data_str.as_bytes();

                    ends_with_newline = data_str.ends_with("\n");
                    handle.write_all(data)?;
                    handle.flush()?;
                }
                if !ends_with_newline {
                    handle.write_all("\n".as_bytes())?;
                }
            }
            ExecutionOutput::StructuredStreamingOutput(mut stream) => {
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                let mut ends_with_newline = false;

                while let Some(res) = stream.sync_next() {
                    let data_str = res?;
                    let data = data_str.as_bytes();

                    ends_with_newline = data_str.ends_with("\n");
                    handle.write_all(data)?;
                    handle.flush()?;
                }
                if !ends_with_newline {
                    handle.write_all("\n".as_bytes())?;
                }
            }
            ExecutionOutput::ImmediateOutput(output) => {
                print!("{}", &output);
                if !output.ends_with("\n") {
                    println!();
                }
            }
            ExecutionOutput::DryRun => {
                println!("[dry run, no llm response]");
            }
            ExecutionOutput::Cached(output) => {
                print!("{}", &output);
                if !output.ends_with("\n") {
                    println!();
                }
            }
            ExecutionOutput::RenderOnly(output) => {
                println!("{}", &output);
            }
        };

        Ok(())
    }
}
