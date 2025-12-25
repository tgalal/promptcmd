use std::io::Write;

use clap::{Parser};
use anyhow::{Result, bail};
use crate::{storage::PromptFilesStorage};


#[derive(Parser)]
pub struct CatCmd {
    #[arg()]
    pub promptname: String,
}

impl CatCmd {

    pub fn exec(&self, storage: &impl PromptFilesStorage,  out: &mut impl Write)-> Result<()> {

        if storage.exists(&self.promptname).is_none() {
            bail!("Could not find a prompt with the name \"{}\"", &self.promptname);
        }

        let promptdata = storage.load(&self.promptname)?.1;

        writeln!(out, "{promptdata}")?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::cmd::cat::CatCmd;
    use crate::storage::promptfiles_mem::InMemoryPromptFilesStorage;
    use crate::{storage::PromptFilesStorage};

    #[test]
    fn test_exec_ok() {
        let mut storage = InMemoryPromptFilesStorage::default();
        storage.store("aaa", "bbbb").unwrap();

        let mut buf = Vec::new();

        let cmd = CatCmd {
            promptname: String::from("aaa")
        };

        cmd.exec(&storage, &mut buf).unwrap();

        assert_eq!(
            buf,
            b"bbbb\n"
        )
    }

    #[test]
    fn test_prompt_not_found() {
        let storage = InMemoryPromptFilesStorage::default();

        let mut buf = Vec::new();

        let cmd = CatCmd {
            promptname: String::from("aaa")
        };

        assert!(cmd.exec(&storage, &mut buf).is_err());
    }
}
