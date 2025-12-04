use serde::{Deserialize, Serialize, Deserializer};
use serde::de::{Error as DeError, Unexpected};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use anyhow::{Context, Result};

pub struct ModelInfo {
    pub model_name: String,
    pub provider: String
}

#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub model: String,
    pub input: Option<Input>,
    pub output: Output
}

#[derive(Debug)]
pub struct InputSchemaElement {
    pub key: String,
    pub data_type:  String,
    pub description: String,
    pub required: bool
} 

#[derive(Debug, Deserialize, Serialize)]
pub struct Input {
    pub schema: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct Output {
    pub format: String,
    pub schema: Option<HashMap<String, String>>,
}

#[derive(Debug)]
pub struct DotPrompt {
    pub frontmatter: Frontmatter,
    pub template: String
}


/// A simple parser error type for TryFrom
#[derive(Debug)]
pub struct ParseDotPromptError(pub String);

impl fmt::Display for ParseDotPromptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseDotPromptError: {}", self.0)
    }
}
impl Error for ParseDotPromptError {}

/// TryFrom<String> implements the actual parsing from raw text -> DotPrompt
impl TryFrom<String> for DotPrompt {
    type Error = ParseDotPromptError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        // Trim left whitespace so files that start with BOM/newlines are handled
        let s = s.trim_start().to_string();

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
            .map_err(|e| ParseDotPromptError(format!("invalid YAML frontmatter: {}", e)))?;

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

    pub fn input_schema(&self) -> HashMap<String, InputSchemaElement> {
        let mut result: HashMap<String, InputSchemaElement> = HashMap::new();

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
                let (sanitized_key, required) = match key.strip_suffix("?") {
                    Some(s) => (s.to_string(), false),
                    None => (key.to_string(), true)
                };
                let (data_type, description) = value.split_once(",")
                    .unwrap_or((value, ""));

                let input_schema_element = InputSchemaElement {
                    key: sanitized_key.to_string(),
                    required,
                    description: description.to_string(),
                    data_type: data_type.to_string()
                };
                result.insert(sanitized_key.to_string(), input_schema_element);
            }
        }
        result
    }
}

// impl<'de> Deserialize<'de> for DotPrompt {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         // Step 1: Deserialize entire input as a raw String
//         let s = String::deserialize(deserializer)?;
//         let trimmed = s.trim();
// 
//         // Validate starting delimiter
//         if !trimmed.starts_with("---") {
//             return Err(D::Error::custom("DotPrompt must start with '---'"));
//         }
// 
//         // Step 2: Split on the first two occurrences of ---
//         let mut split = trimmed.splitn(3, "---");
// 
//         split.next(); // empty before first ---
// 
//         let fm_str = split
//             .next()
//             .ok_or_else(|| D::Error::custom("Missing frontmatter section"))?
//             .trim();
// 
//         let template = split
//             .next()
//             .ok_or_else(|| D::Error::custom("Missing template section after frontmatter"))?
//             .trim()
//             .to_string();
// 
//         // Step 3: Deserialize the YAML frontmatter block
//         let fm: Frontmatter = serde_yaml::from_str(fm_str)
//             .map_err(|e| D::Error::custom(format!("Invalid YAML frontmatter: {e}")))?;
// 
//         Ok(DotPrompt {
//             frontmatter: fm,
//             template,
//         })
//     }
// }
