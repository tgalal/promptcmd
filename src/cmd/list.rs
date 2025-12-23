use clap::{Parser};
use anyhow::{ Result};
use prettytable::{row, Table};
use prettytable::format;
use std::fs;
use crate::config::{appconfig_locator, promptfile_locator};


#[derive(Parser)]
pub struct ListCmd {
    #[arg(short, long)]
    pub long: bool,

    #[arg(short, long)]
    pub enabled: bool,

    #[arg(short, long)]
    pub disabled: bool,

    #[arg(short, long)]
    pub fullpath: bool,
 
    #[arg(short, long)]
    pub prompts: bool,
 
    #[arg(short, long)]
    pub commands: bool,

    /// List lookup paths for configuration files
    #[arg(long)]
    pub config: bool,
}

pub fn exec(
    long: bool,
    enabled: bool,
    disabled: bool,
    fullpath: bool,
    commands: bool,
    config: bool
) -> Result<()> {
    if config {
        exec_for_config()
    } else if commands {
        exec_for_commands(long, fullpath)
    } else {
        exec_for_prompts(long, enabled, disabled, fullpath, commands, config)
    }
}

fn exec_for_config() -> Result<()> {
    let paths : Vec<String>= appconfig_locator::search_paths()
        .iter().map(|path| path.display().to_string()).collect();
    println!("{}", paths.join("\n"));
    Ok(())
}

fn exec_for_prompts(
    long: bool,
    _enabled: bool,
    _disabled: bool,
    _fullpath: bool,
    _commands: bool,
    _config: bool
) -> Result<()> {
    let paths = promptfile_locator::search_paths(None)?;

    let mut promptfiles = Vec::new();

    for path in paths {
        if path.exists() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(e) = path.extension() {
                        if e == "prompt" {
                            promptfiles.push(path);
                        }
                    }
                }

            }
        }
    }

    let promptnames: Vec<&str> = promptfiles.iter().filter_map(|item| item.file_stem()?.to_str()).collect();

    if long {
        
        let outputline: Vec<(String, String)> = promptfiles.iter().filter_map(|item| {
            item.file_stem().map(|promptname| {
                (promptname.to_string_lossy().into_owned(), item.display().to_string()) 
            })
        }).collect();

        let mut table = Table::new(); 
        let format = format::FormatBuilder::new()
            .padding(0, 5)
            .build();
        table.set_format(format);

        for line in outputline {
            table.add_row(row![line.0, line.1]);
        }
        table.printstd();

    } else {
        println!("{}", promptnames.join(" "));
    }

    Ok(())

}

fn exec_for_commands(
    _long: bool,
    _fullpath: bool,
) -> Result<()> {
    Ok(())
}

