use clap::{Arg, Command};
use serde::de::{Error as DeError, Unexpected};
use crate::dotprompt::dotprompt::DotPrompt;
use std::error::Error;
use std::fmt;
use std::convert::TryFrom;

/// A simple parser error type for TryFrom
#[derive(Debug)]
pub struct ArgsFromDotPromptError(pub String);

impl fmt::Display for ArgsFromDotPromptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArgsFromDotPromptError: {}", self.0)
    }
}
impl Error for ArgsFromDotPromptError {}

impl TryFrom<&DotPrompt> for Vec<Arg> {

    type Error = ArgsFromDotPromptError;

    fn try_from(dotprompt: &DotPrompt) -> Result<Self, ArgsFromDotPromptError> {

        let inputschema = dotprompt.input_schema();
        let mut args: Vec<Arg> = Vec::new();

        for (_, inputschema_element) in inputschema {
            let mut arg =  Arg::new(inputschema_element.key.clone())
                .help(inputschema_element.description.clone())
                .required(inputschema_element.required);

            arg = if inputschema_element.data_type == "boolean" {
                arg.long(inputschema_element.key.clone())
                    .action(clap::ArgAction::SetTrue)
            } else if inputschema_element.data_type == "string" {
                if inputschema_element.positional {
                        if inputschema_element.required {
                            arg.num_args(1..)
                        } else {
                            arg.num_args(0..)
                        }
                } else {
                    arg.long(inputschema_element.key.clone())
                }
            } else {
                // bail!("Unsupported data type {} for {}", &inputschema_element.data_type, &inputschema_element.key);
                return Err(ArgsFromDotPromptError(String::from("ERROR")));
            };
            args.push(arg);
        }
        Ok(args)
    }
}
