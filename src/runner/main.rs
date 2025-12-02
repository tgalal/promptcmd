use clap::{Arg, Command};
use std::{env, path::Path};
use anyhow::{Context, Result};
use std::path::PathBuf;

mod config;
use crate::config::locator;

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


    let matches = command.get_matches();

    Ok(())

    // let matches = Command::new("myapp")
    //     .version("1.0")
    //     .about("Does awesome things")
    //     .arg(
    //         Arg::new("name")
    //             .short('n')
    //             .long("name")
    //             .value_name("NAME")
    //             .help("Sets a name")
    //             .required(true),
    //     )
    //     .arg(
    //         Arg::new("count")
    //             .short('c')
    //             .long("count")
    //             .value_name("NUMBER")
    //             .help("Number of times to print")
    //             .default_value("1"),
    //     )
    //     .get_matches();

    // let name = matches.get_one::<String>("name").unwrap();
    // let count: usize = matches
    //     .get_one::<String>("count")
    //     .unwrap()
    //     .parse()
    //     .unwrap();

    // for _ in 0..count {
    //     println!("Hello, {}!", name);
    // }
    //
}
