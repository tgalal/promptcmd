use clap::{Parser};
use anyhow::{ Result};
use prettytable::{row, Table};
use prettytable::format;
use crate::storage::{PromptFilesStorage};


#[derive(Parser)]
pub struct ListCmd {
    #[arg(short, long, help="Print in long format")]
    pub long: bool,
}

impl ListCmd {

    pub fn exec(
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


