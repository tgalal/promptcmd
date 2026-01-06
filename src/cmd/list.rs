use clap::{Parser};
use anyhow::{ Result};
use prettytable::{row, Table};
use prettytable::format;
use crate::config::{appconfig_locator};
use crate::storage::{PromptFilesStorage};


#[derive(Parser)]
pub struct ListCmd {
    #[arg(short, long, help="Print in long format")]
    pub long: bool,
    #[arg(long, help="List config.toml lookup paths")]
    pub config: bool,
}

impl ListCmd {

    pub fn exec(
        &self,
        storage: &impl PromptFilesStorage,
    ) -> Result<()> {
        if self.config {
            self.exec_for_config()
        } else {
            self.exec_for_prompts(storage)
        }
    }
    fn exec_for_config(&self) -> Result<()> {
        let paths : Vec<String>= appconfig_locator::search_paths()
            .iter().map(|path| path.display().to_string()).collect();
        println!("{}", paths.join("\n"));
        Ok(())
    }

    fn exec_for_prompts(
        &self,
        storage: &impl PromptFilesStorage,
    ) -> Result<()> {

        let prompts = storage.list()?;

        if self.long {

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
            if !joined.is_empty() {
                println!("{joined}");
            }
        }

        Ok(())

    }
}


