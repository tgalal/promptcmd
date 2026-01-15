use clap::{value_parser, Arg, ArgMatches};
use serde_json::{Value};

use crate::{dotprompt::{renderers::{RenderError}, DotPrompt}, executor::PromptInputs};

pub struct DotPromptArgMatches<'a> {
    pub matches: ArgMatches,
    pub dotprompt: &'a DotPrompt
}

impl<'a> TryFrom<DotPromptArgMatches<'a>> for PromptInputs {
    type Error = RenderError;
    fn try_from(dp_matches: DotPromptArgMatches) -> Result<Self, Self::Error> {

        let mut inputs = PromptInputs::new();
        let dp = dp_matches.dotprompt;

        let matches = &dp_matches.matches;

        let inputschema = &dp.frontmatter.input.schema;

        for ele in inputschema.values() {
            let value = if ele.data_type == "boolean" {
                match matches.get_one::<bool>(&ele.key) {
                    Some(value) => {
                       Value::Bool(*value)
                    },
                    None => {
                       Value::Bool(false)
                    }
                }
            } else if ele.data_type == "integer" {
                match matches.get_one::<i64>(&ele.key) {
                    Some(value) => {
                        Value::from(*value)
                    },
                    None => {
                        Value::from(0)
                    }
                }
            } else if ele.data_type == "number" {
                match matches.get_one::<f32>(&ele.key) {
                    Some(value) => {
                        Value::from(*value)
                    },
                    None => {
                        Value::from(0f32)
                    }
                }
            }
            else if ele.data_type == "string" {
                if ele.positional {
                    match matches.get_many::<String>(&ele.key) {
                        Some(value) => {
                            Value::from(value.cloned().collect::<Vec<_>>().join(" "))
                        },
                        None => {
                            Value::from("")
                        }
                    }
                } else {
                    match matches.get_one::<String>(&ele.key) {
                        Some(value) => {
                            Value::from(value.to_string())
                        },
                        None => {
                            Value::from("")
                        }
                    }
                }
            } else if ele.data_type == "enum" {
                match matches.get_one::<String>(&ele.key) {
                    Some(value) => {
                        Value::from(value.to_string())
                    },
                    None => {
                        Value::from("")
                    }
                }
            }
            else {
                return Err(RenderError::UnsupportedDataType {
                    key: ele.key.clone(), data_type: ele.data_type.clone()})
            };
            inputs.insert(ele.key.clone(), value);
        }
        Ok(inputs)
    }
}

impl TryFrom<&DotPrompt> for Vec<Arg> {

    type Error = RenderError;

    fn try_from(dotprompt: &DotPrompt) -> Result<Self, RenderError> {

        let inputschema = &dotprompt.frontmatter.input.schema;
        let mut args: Vec<Arg> = Vec::new();

        for inputschema_element in inputschema.values() {
            let mut arg =  Arg::new(inputschema_element.key.clone())
                .help(inputschema_element.description.clone())
                .required(inputschema_element.required);

            arg = if inputschema_element.data_type == "boolean" {
                arg.long(inputschema_element.key.clone())
                    .action(clap::ArgAction::SetTrue)
                    .required(false)
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
            } else if inputschema_element.data_type == "integer" {
                arg.long(inputschema_element.key.clone())
                    .value_parser(value_parser!(i64))
            } else if inputschema_element.data_type == "number" {
                arg.long(inputschema_element.key.clone())
                    .value_parser(value_parser!(f32))
            } else if inputschema_element.data_type == "enum" {
                arg.long(inputschema_element.key.clone())
                    .value_parser(inputschema_element.choices.clone())
            }
            else {
                return Err(
                    RenderError::UnsupportedDataType {
                            key: inputschema_element.key.clone(),
                            data_type: inputschema_element.data_type.clone()
                        }
                    );
            };
            args.push(arg);
        }
        Ok(args)
    }
}
