use crate::stats::store::StatsStore;
use clap::{Parser};
use anyhow::Result;
use prettytable::{row, Table};
use prettytable::format::{self};

#[derive(Parser)]
pub struct StatsCmd;

impl StatsCmd {
    // Group by provider and model
    // Provider     Model   Runs    Total In    Total Out
    pub fn exec(&self, store: &impl StatsStore) -> Result<()> {
        let summary = store.summary(None, None, None, None, None)?;

        let mut table = Table::new();
        let format = format::FormatBuilder::new()
            .padding(0, 5)
            .build();
        table.set_format(format);
        table.add_row(row!["provider", "model", "runs", "prompt tokens", "completion tokens", "avg tps"]);

        for item in summary {
           table.add_row(
                row![item.provider, item.model, item.count, item.prompt_tokens, item.completion_tokens, item.tps]
            );
        }
        table.printstd();
        Ok(())
    }
}

