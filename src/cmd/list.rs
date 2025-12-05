use clap::{Parser};
use log::{debug};
use anyhow::{ Result};
use crate::config::locator;
use prettytable::{row, Table};
use prettytable::format;
use std::fs;


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
}

pub fn exec(
    long: bool,
    enabled: bool,
    disabled: bool,
    fullpath: bool,
    commands: bool
) -> Result<()> {
    if commands {
        exec_for_commands(long, fullpath)
    } else {
        exec_for_prompts(long, enabled, disabled, fullpath, commands)
    }
}

fn exec_for_prompts(
    long: bool,
    enabled: bool,
    disabled: bool,
    fullpath: bool,
    commands: bool
) -> Result<()> {
    // 1. Get all prompts, search all paths
    let paths = locator::get_prompt_search_dirs();

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
    let promptpaths: Vec<String> = promptfiles.iter().map(|item| item.display().to_string()).collect();

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
    long: bool,
    fullpath: bool,
) -> Result<()> {
    Ok(())
}

