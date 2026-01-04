use indenter::indented;
use std::fmt::{self, Write};

use crate::resolver::{base::Base, variant::Variant, ResolvedConfig, ResolvedProperty, ResolvedPropertySource, ResolvedProviderConfig};

impl fmt::Display for ResolvedPropertySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Group(group_name, value) => write!(f, "Group({group_name},{value})"),
            Self::Variant(name) => write!(f, "Variant({name})"),
            Self::Base(name) => write!(f, "Base({name})"),
            Self::Env(name) => write!(f, "Env({name})"),
            Self::Default => write!(f, "Default"),
            Self::Dotprompt(name) => write!(f, "Frontmatter({name})"),
            Self::Input(name) => write!(f, "Input({name})"),
            Self::Other(name) => write!(f, "Other({name})")
        }
    }
}

impl fmt::Display for ResolvedConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolvedConfig::Base(base) => {
                write!(f, "{base}")
            },
            ResolvedConfig::Variant(variant) => {
                write!(f, "{variant}")

            },
            ResolvedConfig::Group(group) => {
                write!(f, "{group}")
            },
        }
    }
}

impl fmt::Display for ResolvedProviderConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Provider: ")?;
        match self {
            Self::Ollama(conf) => {
                writeln!(f, "Ollama")?;
                writeln!(f, "Configuration:")?;
                write!(indented(f).with_str("  "), "{conf}")?
            },
            Self::Anthropic(conf) => {
                writeln!(f, "Anthropic")?;
                writeln!(f, "Configuration:")?;
                write!(indented(f).with_str("  "), "{conf}")?
            },
            Self::OpenAI(conf) => {
                writeln!(f, "OpenAI")?;
                writeln!(f, "Configuration:")?;
                write!(indented(f).with_str("  "), "{conf}")?
            }
        };

        Ok(())
    }
}

impl<T:fmt::Display> fmt::Display for ResolvedProperty<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for Base {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Base: {}", &self.name)?;
        write!(f, "{}", self.resolved)
    }
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Variant: {}", &self.name)?;
        writeln!(f, "Base: {}", self.base_name)?;
        write!(f, "{}", self.resolved)
    }
}
