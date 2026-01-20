use clap::{Parser};
use clap::{Arg, Command};
use handlebars::HelperDef;
use std::collections::HashMap;
use std::io::{BufReader, Write};
use std::sync::{Mutex};
use std::convert::TryFrom;
use anyhow::{Context, Result};
use thiserror::Error;
use crate::cmd::{TextEditor, TextEditorFileType};
use crate::dotprompt::renderers::argmatches::DotPromptArgMatches;
use crate::dotprompt::renderers::Render;
use crate::executor::{PromptInputs};
use crate::dotprompt::{ helpers, DotPrompt};
use crate::storage::PromptFilesStorage;

#[derive(Parser)]
pub struct RenderCmd {
    #[arg()]
    pub promptname: String,

    #[arg(long, short, help="Edit" )]
    pub edit: bool,

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
pub enum RenderCmdError {
    #[error("'{0}' is required but not configured")]
    RequiredConfiguration(&'static str)
}

impl RenderCmd {
    pub fn exec(&self,
        storage: &impl PromptFilesStorage,
        out: &mut impl Write,
        editor: &impl TextEditor
    )-> Result<()> {

        let (_, data) = storage.load(&self.promptname)?;
        let dotprompt = DotPrompt::try_from((self.promptname.as_str(), data.as_str()))?;

        let mut command: Command = Command::new(self.promptname.to_string());

        command = generate_arguments_from_dotprompt(command, &dotprompt)?;

        let params = [vec!["--".to_string()], self.prompt_args.clone()].concat();
        let matches = command.get_matches_from(params);

        let argmatches = DotPromptArgMatches {
            matches,
            dotprompt: &dotprompt
        };

        let inputs: PromptInputs = argmatches.try_into()?;

        let exec_helper: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::ExecHelper);
        let concat_helper: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::ConcatHelper);
        let stdin_helper: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::StdinHelper {
            inp: Mutex::new(BufReader::new(std::io::stdin()))
        });
        let stdin_helper2: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::StdinHelper {
            inp: Mutex::new(BufReader::new(std::io::stdin()))
        });
        let ask_helper: Box<dyn HelperDef + Send + Sync> = Box::new(helpers::AskHelper {
            promptname: dotprompt.name.clone(),
            inp:Mutex::new(BufReader::new(std::io::stdin()))
        });

        let helpers_map:HashMap<&str, Box<dyn HelperDef + Send + Sync>> = HashMap::from([
            ("exec", exec_helper),
            ("concat", concat_helper),
            ("stdin", stdin_helper),
            ("STDIN", stdin_helper2),
            ("ask", ask_helper),
        ]);

        let mut rendered_dotprompt: String = dotprompt.render(inputs, helpers_map)?;

        if self.edit {
            rendered_dotprompt = editor.edit(&rendered_dotprompt, TextEditorFileType::Dotprompt)?;
        }

        write!(out, "{}", &rendered_dotprompt)?;

        Ok(())
    }
}
