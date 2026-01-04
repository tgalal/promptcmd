use clap::{Parser};
use anyhow::{ Result};
use prettytable::{row, Table};
use prettytable::format;
use crate::config::{appconfig_locator};
use crate::storage::{PromptFilesStorage};


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
    storage: &impl PromptFilesStorage,
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
        exec_for_prompts(storage, long, enabled, disabled, fullpath, commands, config)
    }
}

fn exec_for_config() -> Result<()> {
    let paths : Vec<String>= appconfig_locator::search_paths()
        .iter().map(|path| path.display().to_string()).collect();
    println!("{}", paths.join("\n"));
    Ok(())
}

fn exec_for_prompts(
    storage: &impl PromptFilesStorage,
    long: bool,
    _enabled: bool,
    _disabled: bool,
    _fullpath: bool,
    _commands: bool,
    _config: bool
) -> Result<()> {

    let prompts = storage.list()?;

    if long {

        let mut table = Table::new();
        let format = format::FormatBuilder::new()
            .padding(0, 5)
            .build();
        table.set_format(format);

        for (identifier, path) in prompts {
            table.add_row(row![identifier, path]);
        }

        table.printstd();

    } else {
        let joined = prompts.keys().cloned().collect::<Vec<_>>().join(" ");
        println!("{joined}");
    }

    Ok(())

}

fn exec_for_commands(
    _long: bool,
    _fullpath: bool,
) -> Result<()> {
    Ok(())
}

