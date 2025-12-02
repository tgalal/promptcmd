use anyhow::{Result, anyhow, bail};
use clap::{Arg, Args, Command, Parser, Subcommand};
use std::fs;
use crate::config::ConfigLocator;

use crate::dotprompt::{Frontmatter};

#[derive(Subcommand)]
enum ReadSubcommands {
    Translate(TranslateCmd),
}

#[derive(Parser)]
struct TranslateCmd {
    #[arg(long)]
    arg1: bool,

    #[arg(long)]
    arg2: bool,
}

#[derive(Parser)]
// #[clap(allow_hyphen_values = true)]
pub struct ReadCmd {
    #[arg(required=true)]
    command: String,
    // #[arg(short, long, required=false)]
    // help_abc: bool,
    // #[clap(allow_hyphen_values = true)]
    #[arg( trailing_var_arg=true)]
    rest: Vec<String>,
}

#[derive(Debug, Clone, Parser, PartialEq)]
#[clap(version)]
struct Cli {
    // #[command(subcommand)]
    // command: Commands,

    #[arg(short, long)]
    verbose: bool
}



pub fn exec(cmd: ReadCmd) -> Result<()> {
    println!("Got command: {}", cmd.command);
    // println!("With Res: {}", cmd.rest);
    // let target_cmd: &str = &cmd.command;
    // let config_filename = String::from(target_cmd) + ".prompt";
    // let config_filename =  cmd.command + ".prompt";
    let target_command: &String = &cmd.command;
    let config_filename: String =  format!("{target_command}.prompt");

    let locator: ConfigLocator = ConfigLocator::new("aibox", "prompts.d", config_filename);

    println!("Searching for config in:");
    for path in locator.get_search_paths() {
        println!("  - {}", path.display());
    }

    match locator.find_config() {
        Some(path) => {
            println!("\nFound config at: {}", path.display());
            let content = fs::read_to_string(path)?;
            let parts: Vec<&str> = content.split("---").collect();
            let yaml_content = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("No frontmatter found"))?;

            let frontmatter: Frontmatter = serde_yaml::from_str(yaml_content)?;
            // println!("{:#?}", frontmatter.model);
            // println!("{frontmatter}");


            // let target_cmd: &str = cmd.command.as_str();
            let mut command_args = Command::new(target_command);

            if let Some(schema) = frontmatter.input.schema {
                // println!("Schema:");
                for (key, value) in schema {
                    // println!("{key}: {value}");

                    command_args = command_args.arg(
                        Arg::new(&key)
                            .long(&key)
                            .help(&value));
                    // let mut cli = Command::new("pipelight");
                    // cli = Cli::augment_args(cli);
                    // cli = cli
                    //     .mut_subcommand("logs", |a| {
                    //         a.mut_arg("color", |e| {
                    //             e.num_args(0..=1)
                    //                 .require_equals(true)
                    //                 .default_missing_value("always")
                    //                 .default_value("auto")
                    //         })
                    //     });
                    // let matches = cli.get_matches();


                }
            }

            if cmd.rest.contains(&String::from("--help")) {
                command_args.print_help().unwrap();
            }
            let matches = command_args.get_matches_from(&cmd.rest);
            // command_args.print_help().unwrap();
            // let matches = command_args.get_matches_from(&cmd.rest);
            // if cmd.rest.contains(&"--dynamic-help".to_string()) { 
            // }

        },
        None => println!("\nNo config file found"),
    }

    Ok(())
}
