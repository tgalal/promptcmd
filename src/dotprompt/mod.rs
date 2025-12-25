pub mod args;
pub mod render;
pub mod installer;

use serde::{Deserialize, Serialize};
use std::{collections::HashMap};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use anyhow::{Context, Result};
use serde_json::json;

pub struct ModelInfo {
    pub model_name: String,
    pub provider: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct Frontmatter {
    pub model: String,
    pub input: Option<Input>,
    pub output: Option<Output>
}

#[derive(Debug)]
pub struct SchemaElement {
    pub key: String,
    pub data_type:  String,
    pub description: String,
    pub required: bool,
    pub positional: bool
} 

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    pub schema: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Output {
    pub format: String,
    pub schema: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct DotPrompt {
    pub frontmatter: Frontmatter,
    pub template: String
}

#[derive(Debug)]
pub struct ParseDotPromptError(pub String);

impl fmt::Display for ParseDotPromptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseDotPromptError: {}", self.0)
    }
}

impl Error for ParseDotPromptError {}

impl TryFrom<&str> for DotPrompt {
    type Error = ParseDotPromptError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        // Skip all lines starting with '#'
        let s = s.trim_start().lines()
            .skip_while(
                |line| {
                    let trimmed = line.trim();
                    trimmed.starts_with("#") || trimmed.is_empty()
                }
            )
            .collect::<Vec<_>>()
            .join("\n");

        // Expect frontmatter to start with `---`
        if !s.starts_with("---") {
            return Err(ParseDotPromptError(
                "input must start with frontmatter delimiter `---`".into(),
            ));
        }

        // Split into at most 3 parts: before first ---, frontmatter, rest.
        // Using splitn(3, "---") yields: ["", "<fm>", "<rest>"]
        let mut parts = s.splitn(3, "---");

        // skip prefix (could be empty)
        parts.next();

        let fm_str = parts
            .next()
            .ok_or_else(|| ParseDotPromptError("missing frontmatter block".into()))?
            .trim();

        let template = parts
            .next()
            .ok_or_else(|| ParseDotPromptError("missing template after frontmatter".into()))?
            .trim()
            .to_string();

        // Now parse the frontmatter YAML into the typed struct
        let fm: Frontmatter = serde_yaml::from_str(fm_str)
            .map_err(|e| ParseDotPromptError(format!("invalid YAML frontmatter: {e}")))?;

        Ok(DotPrompt {
            frontmatter: fm,
            template,
        })
    }
}

impl DotPrompt {
    pub fn template_needs_stdin(&self) -> bool {
        self.template.contains("{{STDIN}}")
    }

    pub fn model_info(&self) -> Result<ModelInfo> {
        let dissected = self.frontmatter.model.split_once("/").context("Improper model format")?;
        Ok(ModelInfo {
            model_name: dissected.1.to_string(),
            provider: dissected.0.to_string()
        })
    }

    pub fn output_format(&self) -> String {
        if let Some(ref output) = self.frontmatter.output {
            return output.format.clone();
        }

        String::from("text")
    }

    pub fn output_to_extract_structured_json(&self, name: &str) -> String {
        let output_schema = self.output_schema();
        let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
        let mut required: Vec<String> = Vec::new();

        for (_, element) in output_schema {
            let json_data_type: &str = if element.data_type == "number" {
                "number"
            } else if element.data_type == "boolean" {
                "boolean"
            } else {
                "string"
            };
            let json_value = json!({
                "type": json_data_type,
                "description": element.description
            });
            properties.insert(element.key.clone(), json_value);

            if element.required {
                required.push(element.key.clone());
            }
        }
        let result2 = json!({
            "name": name,
            "schema": {
                "type": "object",
                "properties": properties,
                "required": required
            }
        });

        result2.to_string()
    }

    pub fn output_schema(&self) -> HashMap<String, SchemaElement> {
        let mut result: HashMap<String, SchemaElement> = HashMap::new();

        let schema = match &self.frontmatter.output {
            Some(output) => {
                &output.schema
            }
            None => {
                return result;
            }
        };

        if let Some(schema) = schema {
            for (key, value) in schema {
                let mut key_chars = key.chars();

                let required = if key.ends_with("?") {
                    key_chars.next_back();
                    false
                } else {
                    true
                };

                let sanitized_key = key_chars.as_str();

                let (data_type, description) = value.split_once(",")
                    .unwrap_or((value, ""));

                let input_schema_element = SchemaElement {
                    key: sanitized_key.to_string(),
                    required,
                    description: description.to_string(),
                    data_type: data_type.to_string(),
                    positional: false
                };
                result.insert(sanitized_key.to_string(), input_schema_element);
            }
        }
        result
    }

    pub fn input_schema(&self) -> HashMap<String, SchemaElement> {
        let mut result: HashMap<String, SchemaElement> = HashMap::new();

        let input = &self.frontmatter.input;

        let schema = match input {
            Some(input) => {
                &input.schema
            }
            None => {
                return result;
            }
        };


        if let Some(input_schema) = schema {
            for (key, value) in input_schema {
                let mut key_chars = key.chars();
                let (required, positional) = if key.ends_with("?!") || key.ends_with("!?") {
                    // optional and positional
                    key_chars.next_back();
                    key_chars.next_back();
                    (false, true)
                } else if key.ends_with("?") {
                    // optional
                    key_chars.next_back();
                    (false, false)
                } else if key.ends_with("!") {
                    // positional
                    key_chars.next_back();
                    (true, true)
                } else {
                    (true, false)
                };

                let sanitized_key = key_chars.as_str();

                let (data_type, description) = value.split_once(",")
                    .unwrap_or((value, ""));

                let input_schema_element = SchemaElement {
                    key: sanitized_key.to_string(),
                    required,
                    description: description.to_string(),
                    data_type: data_type.to_string(),
                    positional
                };
                result.insert(sanitized_key.to_string(), input_schema_element);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_dotprompt() {
        let content = r#"---
model: anthropic/claude-3-5-sonnet-20241022
input:
  schema:
    message: string, The message to translate
output:
  format: text
---
Translate this: {{message}}"#;

        let result = DotPrompt::try_from(content);
        assert!(result.is_ok(), "Should parse valid dotprompt");

        let dotprompt = result.unwrap();
        assert_eq!(dotprompt.frontmatter.model, "anthropic/claude-3-5-sonnet-20241022");
        assert_eq!(dotprompt.template, "Translate this: {{message}}");
        assert_eq!(dotprompt.frontmatter.output.unwrap().format, "text");
    }

    #[test]
    fn test_parse_with_comments() {
        let content = r#"# This is a comment
# Another comment

---
model: ollama/llama3
input:
  schema:
    query: string
output:
  format: text
---
Query: {{query}}"#;

        let result = DotPrompt::try_from(content);
        assert!(result.is_ok(), "Should skip comments and parse successfully");

        let dotprompt = result.unwrap();
        assert_eq!(dotprompt.frontmatter.model, "ollama/llama3");
    }

    #[test]
    fn test_parse_without_frontmatter_delimiter() {
        let content = r#"model: test/model
output:
  format: text"#;

        let result = DotPrompt::try_from(content);
        assert!(result.is_err(), "Should fail without frontmatter delimiter");

        if let Err(e) = result {
            assert!(e.0.contains("frontmatter delimiter"));
        }
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let content = r#"---
model: test
this is not valid yaml: [
output:
  format: text
---
Template"#;

        let result = DotPrompt::try_from(content);
        assert!(result.is_err(), "Should fail on invalid YAML");

        if let Err(e) = result {
            assert!(e.0.contains("invalid YAML"));
        }
    }

    #[test]
    fn test_parse_missing_template() {
        let content = r#"---
model: test/model
output:
  format: text
---"#;

        let result = DotPrompt::try_from(content);
        // This should actually succeed with an empty template
        assert!(result.is_ok());
        assert_eq!(result.unwrap().template, "");
    }

    #[test]
    fn test_input_schema_required_field() {
        let content = r#"---
model: test/model
input:
  schema:
    name: string, User's name
output:
  format: text
---
Hello {{name}}"#;

        let dotprompt = DotPrompt::try_from(content).unwrap();
        let schema = dotprompt.input_schema();

        assert_eq!(schema.len(), 1);
        let name_field = schema.get("name").unwrap();
        assert_eq!(name_field.key, "name");
        assert_eq!(name_field.data_type, "string");
        assert_eq!(name_field.description, " User's name");
        assert!(name_field.required, "Should be required by default");
        assert!(!name_field.positional, "Should not be positional by default");
    }

    #[test]
    fn test_input_schema_optional_field() {
        let content = r#"---
model: test/model
input:
  schema:
    "age?": string, Optional age
output:
  format: text
---
Age: {{age}}"#;

        let dotprompt = DotPrompt::try_from(content).unwrap();
        let schema = dotprompt.input_schema();

        let age_field = schema.get("age").unwrap();
        assert_eq!(age_field.key, "age");
        assert!(!age_field.required, "Should be optional");
        assert!(!age_field.positional);
    }

    #[test]
    fn test_input_schema_positional_field() {
        let content = r#"---
model: test/model
input:
  schema:
    "files!": string, Files to process
output:
  format: text
---
Files: {{files}}"#;

        let dotprompt = DotPrompt::try_from(content).unwrap();
        let schema = dotprompt.input_schema();

        let files_field = schema.get("files").unwrap();
        assert_eq!(files_field.key, "files");
        assert!(files_field.required, "Should be required");
        assert!(files_field.positional, "Should be positional");
    }

    #[test]
    fn test_input_schema_optional_positional() {
        let content = r#"---
model: test/model
input:
  schema:
    args?!: string, Optional positional args
output:
  format: text
---
Args: {{args}}"#;

        let dotprompt = DotPrompt::try_from(content).unwrap();
        let schema = dotprompt.input_schema();

        let args_field = schema.get("args").unwrap();
        assert_eq!(args_field.key, "args");
        assert!(!args_field.required, "Should be optional");
        assert!(args_field.positional, "Should be positional");
    }

    #[test]
    fn test_input_schema_optional_positional_reverse() {
        let content = r#"---
model: test/model
input:
  schema:
    args!?: string, Optional positional args (reverse syntax)
output:
  format: text
---
Args: {{args}}"#;

        let dotprompt = DotPrompt::try_from(content).unwrap();
        let schema = dotprompt.input_schema();

        let args_field = schema.get("args").unwrap();
        assert_eq!(args_field.key, "args");
        assert!(!args_field.required, "Should be optional");
        assert!(args_field.positional, "Should be positional");
    }

    #[test]
    fn test_input_schema_no_input() {
        let content = r#"---
model: test/model
output:
  format: text
---
No input needed"#;

        let dotprompt = DotPrompt::try_from(content).unwrap();
        let schema = dotprompt.input_schema();

        assert_eq!(schema.len(), 0, "Should return empty schema");
    }

    #[test]
    fn test_input_schema_multiple_fields() {
        let content = r#"---
model: test/model
input:
  schema:
    name: string, User name
    age?: string, User age
    files!: string, Files to process
    verbose?: boolean, Verbose output
output:
  format: text
---
Template"#;

        let dotprompt = DotPrompt::try_from(content).unwrap();
        let schema = dotprompt.input_schema();

        assert_eq!(schema.len(), 4);
        assert!(schema.contains_key("name"));
        assert!(schema.contains_key("age"));
        assert!(schema.contains_key("files"));
        assert!(schema.contains_key("verbose"));
    }

    #[test]
    fn test_template_needs_stdin_true() {
        let content = r#"---
model: test/model
output:
  format: text
---
Process this: {{STDIN}}"#;

        let dotprompt = DotPrompt::try_from(content).unwrap();
        assert!(dotprompt.template_needs_stdin());
    }

    #[test]
    fn test_template_needs_stdin_false() {
        let content = r#"---
model: test/model
output:
  format: text
---
No stdin needed"#;

        let dotprompt = DotPrompt::try_from(content).unwrap();
        assert!(!dotprompt.template_needs_stdin());
    }

    #[test]
    fn test_model_info_parsing() {
        let content = r#"---
model: anthropic/claude-3-5-sonnet-20241022
output:
  format: text
---
Template"#;

        let dotprompt = DotPrompt::try_from(content).unwrap();
        let model_info = dotprompt.model_info();

        assert!(model_info.is_ok());
        let info = model_info.unwrap();
        assert_eq!(info.provider, "anthropic");
        assert_eq!(info.model_name, "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_model_info_invalid_format() {
        let content = r#"---
model: invalid-model-format
output:
  format: text
---
Template"#;

        let dotprompt = DotPrompt::try_from(content).unwrap();
        let model_info = dotprompt.model_info();

        assert!(model_info.is_err(), "Should fail on invalid model format");
    }

    #[test]
    fn test_schema_without_description() {
        let content = r#"---
model: test/model
input:
  schema:
    name: string
output:
  format: text
---
Template"#;

        let dotprompt = DotPrompt::try_from(content).unwrap();
        let schema = dotprompt.input_schema();

        let name_field = schema.get("name").unwrap();
        assert_eq!(name_field.data_type, "string");
        assert_eq!(name_field.description, "");
    }

    #[test]
    fn test_complex_template() {
        let content = r#"---
model: test/model
input:
  schema:
    name: string
    language: string
output:
  format: text
---
Hello {{name}}!
Please translate to {{language}}.
Multiple lines supported."#;

        let dotprompt = DotPrompt::try_from(content).unwrap();
        assert!(dotprompt.template.contains("Hello {{name}}"));
        assert!(dotprompt.template.contains("{{language}}"));
        assert!(dotprompt.template.contains("Multiple lines"));
    }
}

