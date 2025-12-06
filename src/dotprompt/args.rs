use clap::{Arg, ArgMatches};
use crate::dotprompt::{DotPrompt, render::Render};
use std::collections::HashMap;
use std::convert::TryFrom;
use thiserror::Error;
use handlebars::{Handlebars, RenderError, TemplateError};

#[derive(Error, Debug)]
pub enum DotPromptArgsError {
    #[error("Failed to generate arguments from schema, reason: {0}")]
    ArgsFromDotPromptError(String),
    #[error("Unsupported data type {key}, {data_type}")]
    UnsupportedDataType {
        key: String,
        data_type: String
    },
    #[error("Rendering handlebars template failed")]
    HandlebarsRegisterTemplateError(#[from] TemplateError),

    #[error("data store disconnected")]
    HandlebarsRenderTemplateError(#[from] RenderError),
}


impl TryFrom<&DotPrompt> for Vec<Arg> {

    type Error = DotPromptArgsError;

    fn try_from(dotprompt: &DotPrompt) -> Result<Self, DotPromptArgsError> {

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

                return Err(
                    DotPromptArgsError::UnsupportedDataType {
                            key: inputschema_element.key,
                            data_type: inputschema_element.data_type
                        }
                    );
            };
            args.push(arg);
        }
        Ok(args)
    }
}

impl Render<ArgMatches, DotPromptArgsError> for DotPrompt {
    fn render(&self, matches: &ArgMatches, extra: &HashMap<String, String>) -> Result<String, DotPromptArgsError> {

        let mut handlebar_maps: HashMap<String, String> = extra.clone();

        let inputschema = self.input_schema();

        for (_, ele) in inputschema {
            let value = if ele.data_type == "boolean" {
                match matches.get_one::<bool>(&ele.key) {
                    Some(value) => {
                        value.to_string()
                    },
                    None => {
                        String::from("")
                    }
                }
            } else if ele.data_type == "string" {
                if ele.positional {
                    match matches.get_many::<String>(&ele.key) {
                        Some(value) => {
                            value.cloned().collect::<Vec<_>>().join(" ")
                        },
                        None => {
                            String::from("")
                        }
                    }
                } else {
                    match matches.get_one::<String>(&ele.key) {
                        Some(value) => {
                            value.to_string()
                        },
                        None => {
                            String::from("")
                        }
                    }
                }
            } else {
                return Err(DotPromptArgsError::UnsupportedDataType { key: ele.key, data_type: ele.data_type })
            };
            handlebar_maps.insert(ele.key.to_string(), value);
        }


        let mut hbs = Handlebars::new();
        hbs.register_template_string("prompt", &self.template)?;

        let output = hbs.render("prompt", &handlebar_maps)?;

        Ok(output)
    }
}

// pub trait HandleBarsProcess {
//     fn process_dotprompt(&self, dotprompt: &DotPrompt) -> Result<HashMap<String, String>, ArgsError>;
// }
// 
// #[derive(Debug, PartialEq, Clone)]
// pub enum ArgsError {
//     GeneralError,
// }
// 
// impl HandleBarsProcess for ArgMatches {
//     fn process_dotprompt(&self, dotprompt: &DotPrompt) -> Result<HashMap<String, String>, ArgsError> {
//         let mut handlebar_maps: HashMap<String, String> = HashMap::new();
//         let inputschema = dotprompt.input_schema();
//         let matches= self;
// 
// 
//         for (_, ele) in inputschema {
//             let value = if ele.data_type == "boolean" {
//                 match matches.get_one::<bool>(&ele.key) {
//                     Some(value) => {
//                         value.to_string()
//                     },
//                     None => {
//                         String::from("")
//                     }
//                 }
//             } else if ele.data_type == "string" {
//                 if ele.positional {
//                     match matches.get_many::<String>(&ele.key) {
//                         Some(value) => {
//                             value.cloned().collect::<Vec<_>>().join(" ")
//                         },
//                         None => {
//                             String::from("")
//                         }
//                     }
//                 } else {
//                     match matches.get_one::<String>(&ele.key) {
//                         Some(value) => {
//                             value.to_string()
//                         },
//                         None => {
//                             String::from("")
//                         }
//                     }
//                 }
//             } else {
//                 return Err(ArgsError::GeneralError);
//                 // bail!("Unsupported data type {} for {}", &ele.data_type, &ele.key);
//             };
//             handlebar_maps.insert(ele.key.to_string(), value);
//         }
// 
//         Ok(handlebar_maps)
//     }
// }
