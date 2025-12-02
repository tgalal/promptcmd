use aibox::dotprompt::dotprompt::DotPrompt;
use clap::{Arg, Command};
use std::hash::Hash;
use std::{env, path::Path};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::fs;
use handlebars::Handlebars;
use std::collections::HashMap;
use std::io::{self, Read};

use aibox::config::ConfigLocator;
use aibox::dotprompt::{self, Frontmatter};

const BIN_NAME: &str = "promptbox";

fn main() -> Result<()> {

    let mut args = env::args();

    let path: PathBuf = args
        .next()
        .context("Could not figure binary name")?
        .into();

    let invoked_binname = path
        .file_name()
        .context("Could not get filename")?
        .to_string_lossy()
        .to_string();


    println!("Invoked bin is the following: {invoked_binname}");

    let mut command: Command = Command::new(&invoked_binname)
        .version("1.0");

    let promptname = if invoked_binname == BIN_NAME {
        // Not running: via symlink, first positional argument is the prompt name
        command = command.arg(Arg::new("promptname"));
        args
            .next()
            .context("Could not determine prompt name")?

    } else {
        invoked_binname
    };
    
    println!("Prompt name: {promptname}");

    let config_filename: String =  format!("{promptname}.prompt");
    let locator: ConfigLocator = ConfigLocator::new("aibox", "prompts.d", config_filename);

    println!("Searching for config in:");
    for path in locator.get_search_paths() {
        println!("  - {}", path.display());
    }

    let dotprompt: DotPrompt = match locator.find_config() {
        Some(path) => {
            println!("Loading config from {}", path.display());
            let content = fs::read_to_string(path)?;
            DotPrompt::try_from(content)?
        },
        None => bail!("No config")
    };

    let inputschema = dotprompt.input_schema();

    for (_, inputschema_element) in inputschema {
        let arg = Arg::new(inputschema_element.key.clone())
            .long(inputschema_element.key.clone())
            .help(inputschema_element.description.clone())
            .required(inputschema_element.required);
        command = command.arg(arg);
    }

    let matches = command.get_matches();


    let inputschema = dotprompt.input_schema();
    let mut handlebar_maps: HashMap<String, String> = HashMap::new();
    for (_, ele) in inputschema {
        let value = match matches.get_one::<String>(&ele.key) {
            Some(value) => {
                value.to_string()
            },
            None => {
                String::from("")
            }
        };
        handlebar_maps.insert(ele.key.to_string(), value);
    }

    if dotprompt.template_needs_stdin() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read stdin")?;
        handlebar_maps.insert(String::from("STDIN"), buffer);
    }

    let mut hbs = Handlebars::new();
    hbs.register_template_string("prompt", &dotprompt.template)
        .unwrap();

    let output = hbs.render("prompt", &handlebar_maps)
        .context("Failed to parse template")?;

    println!("{output}");

    Ok(())

}
