use std::io::Write;

use clap::{Parser};
use anyhow::{Result, bail};
use crate::{storage::PromptFilesStorage};


#[derive(Parser)]
pub struct CatCmd {
    #[arg()]
    pub promptname: String,
}

pub fn exec(storage: &impl PromptFilesStorage,  promptname: &str, out: &mut impl Write)-> Result<()> {

    if storage.exists(promptname).is_none() {
        bail!("Could not find a prompt with the name \"{promptname}\"");
    }

    let promptdata = storage.load(promptname)?.1;
    let printable = String::from_utf8_lossy(&promptdata);

    writeln!(out, "{printable}")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::cmd;
    use crate::storage::promptfiles_mem::InMemoryPromptFilesStorage;
    use crate::{storage::PromptFilesStorage};

    #[test]
    fn test_exec_ok() {
        let mut storage = InMemoryPromptFilesStorage::new();
        storage.store("aaa", b"bbbb").unwrap();

        let mut buf = Vec::new();
        cmd::cat::exec(&storage, "aaa", &mut buf).unwrap();

        assert_eq!(
            buf,
            b"bbbb\n"
        )
    }

    #[test]
    fn test_prompt_not_found() {
        let storage = InMemoryPromptFilesStorage::new();

        let mut buf = Vec::new();

        assert!(
            cmd::cat::exec(&storage, "aaa", &mut buf).is_err()
        );
    }
}
