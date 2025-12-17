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
    #[error("Unsupported data type: {data_type} for key: {key}")]
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::dotprompt::DotPrompt;
    use clap::Command;

    fn create_test_dotprompt(content: &str) -> DotPrompt {
        DotPrompt::try_from(content).unwrap()
    }

    #[test]
    fn test_args_from_dotprompt_string_field() {
        let content = r#"---
model: test/model
input:
  schema:
    name: string, User name
output:
  format: text
---
Hello {{name}}"#;

        let dotprompt = create_test_dotprompt(content);
        let args: Result<Vec<Arg>, _> = Vec::try_from(&dotprompt);

        assert!(args.is_ok(), "Should generate args successfully");
        let args = args.unwrap();
        assert_eq!(args.len(), 1);
    }

    #[test]
    fn test_args_from_dotprompt_boolean_field() {
        let content = r#"---
model: test/model
input:
  schema:
    verbose?: boolean, Enable verbose mode
output:
  format: text
---
Verbose: {{verbose}}"#;

        let dotprompt = create_test_dotprompt(content);
        let args: Result<Vec<Arg>, _> = Vec::try_from(&dotprompt);

        assert!(args.is_ok(), "Should generate boolean arg");
        let args = args.unwrap();
        assert_eq!(args.len(), 1);
    }

    #[test]
    fn test_args_from_dotprompt_multiple_fields() {
        let content = r#"---
model: test/model
input:
  schema:
    name: string, User name
    age?: string, User age
    verbose?: boolean, Verbose mode
output:
  format: text
---
Template"#;

        let dotprompt = create_test_dotprompt(content);
        let args: Result<Vec<Arg>, _> = Vec::try_from(&dotprompt);

        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.len(), 3);
    }

    #[test]
    fn test_args_positional_field() {
        let content = r#"---
model: test/model
input:
  schema:
    files!: string, Files to process
output:
  format: text
---
Files: {{files}}"#;

        let dotprompt = create_test_dotprompt(content);
        let args: Result<Vec<Arg>, _> = Vec::try_from(&dotprompt);

        assert!(args.is_ok(), "Should handle positional args");
    }

    #[test]
    fn test_render_with_string_values() {
        let content = r#"---
model: test/model
input:
  schema:
    name: string, User name
    message: string, Message text
output:
  format: text
---
Hello {{name}}, your message: {{message}}"#;

        let dotprompt = create_test_dotprompt(content);
        let args: Vec<Arg> = Vec::try_from(&dotprompt).unwrap();

        // Create a command and parse args
        let cmd = Command::new("test")
            .args(args);

        let matches = cmd.get_matches_from(vec!["test", "--name", "Alice", "--message", "Hi there"]);

        let rendered = dotprompt.render(&matches, &HashMap::new());
        assert!(rendered.is_ok());

        let output = rendered.unwrap();
        assert_eq!(output, "Hello Alice, your message: Hi there");
    }

    #[test]
    fn test_render_with_boolean_values() {
        let content = r#"---
model: test/model
input:
  schema:
    "verbose?": boolean, Verbose mode
output:
  format: text
---
Verbose mode: {{verbose}}"#;

        let dotprompt = create_test_dotprompt(content);
        let args: Vec<Arg> = Vec::try_from(&dotprompt).unwrap();

        let cmd = Command::new("test").args(args);
        let matches = cmd.get_matches_from(vec!["test", "--verbose"]);

        let rendered = dotprompt.render(&matches, &HashMap::new());
        assert!(rendered.is_ok());

        let output = rendered.unwrap();
        assert_eq!(output, "Verbose mode: true");
    }

    #[test]
    fn test_render_with_positional_args() {
        let content = r#"---
model: test/model
input:
  schema:
    positionals!: string, Positional inputs
output:
  format: text
---
Processing: {{positionals}}"#;

        let dotprompt = create_test_dotprompt(content);
        let args: Vec<Arg> = Vec::try_from(&dotprompt).unwrap();

        let cmd = Command::new("test").args(args);
        let matches = cmd.get_matches_from(vec!["test", "pos1", "pos2", "pos3"]);

        let rendered = dotprompt.render(&matches, &HashMap::new());
        assert!(rendered.is_ok());

        let output = rendered.unwrap();
        assert_eq!(output, "Processing: pos1 pos2 pos3");
    }

    #[test]
    fn test_render_with_extra_variables() {
        let content = r#"---
model: test/model
input:
  schema:
    name: string
output:
  format: text
---
Hello {{name}}, extra: {{STDIN}}"#;

        let dotprompt = create_test_dotprompt(content);
        let args: Vec<Arg> = Vec::try_from(&dotprompt).unwrap();

        let cmd = Command::new("test").args(args);
        let matches = cmd.get_matches_from(vec!["test", "--name", "Bob"]);

        let mut extra = HashMap::new();
        extra.insert("STDIN".to_string(), "extra data".to_string());

        let rendered = dotprompt.render(&matches, &extra);
        assert!(rendered.is_ok());

        let output = rendered.unwrap();
        assert_eq!(output, "Hello Bob, extra: extra data");
    }

    #[test]
    fn test_render_optional_field_missing() {
        let content = r#"---
model: test/model
input:
  schema:
    "name?": string, Optional name
output:
  format: text
---
Name: {{name}}"#;

        let dotprompt = create_test_dotprompt(content);
        let args: Vec<Arg> = Vec::try_from(&dotprompt).unwrap();

        let cmd = Command::new("test").args(args);
        let matches = cmd.get_matches_from(vec!["test"]);

        let rendered = dotprompt.render(&matches, &HashMap::new());
        assert!(rendered.is_ok());

        let output = rendered.unwrap();
        // Optional field should be empty string when not provided
        assert_eq!(output, "Name: ");
    }

    #[test]
    fn test_no_input_schema() {
        let content = r#"---
model: test/model
output:
  format: text
---
No input needed"#;

        let dotprompt = create_test_dotprompt(content);
        let args: Result<Vec<Arg>, _> = Vec::try_from(&dotprompt);

        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.len(), 0, "Should generate no args");
    }

    #[test]
    fn test_render_no_input_schema() {
        let content = r#"---
model: test/model
output:
  format: text
---
Static template with no variables"#;

        let dotprompt = create_test_dotprompt(content);
        let args: Vec<Arg> = Vec::try_from(&dotprompt).unwrap();

        let cmd = Command::new("test").args(args);
        let matches = cmd.get_matches_from(vec!["test"]);

        let rendered = dotprompt.render(&matches, &HashMap::new());
        assert!(rendered.is_ok());

        let output = rendered.unwrap();
        assert_eq!(output, "Static template with no variables");
    }
}
