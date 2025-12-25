use clap::{Parser};
use anyhow::{bail, Result};

use crate::cmd::create::{validate_and_write, WriteResult};
use crate::cmd::TextEditor;
use crate::config::appconfig::AppConfig;
use crate::storage::PromptFilesStorage;

#[derive(Parser)]
pub struct EditCmd {
    #[arg(short, long, default_value_t=false)]
    pub force: bool,

    #[arg()]
    promptname: String,
}

pub fn exec(
    inp: &mut impl std::io::BufRead,
    out: &mut impl std::io::Write,
    storage: &mut impl PromptFilesStorage,
    editor: &impl TextEditor,
    appconfig: &AppConfig,
    cmd: EditCmd) -> Result<()> {

    let promptname = cmd.promptname;

    if storage.exists(&promptname).is_some() {
        let (path, content) = storage.load(&promptname)?;
        let mut edited = content.clone();
        println!("Editing {path}");
        loop {
            edited = editor.edit(&edited)?;
            if content != edited {
                match validate_and_write(
                    inp,
                    storage, appconfig, &promptname,
                    edited.as_str(), cmd.force)? {

                    WriteResult::Validated(_, path) | WriteResult::Written(path)=> {
                        writeln!(out, "Saved {path}")?;
                        break;
                    }
                    WriteResult::Aborted => {
                        println!("Editing aborted, no changes were saved");
                        break;
                    }
                    WriteResult::Edit => {}
                }
            } else {
                println!("No changes");
                break;
            }
        }
    } else {
        bail!("Could not find prompt file");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{cmd::{self, edit::EditCmd, TextEditor}, config::appconfig::AppConfig, installer::{tests::InMemoryInstaller, DotPromptInstaller}, storage::{promptfiles_mem::InMemoryPromptFilesStorage, PromptFilesStorage}};

    const PROMPTFILE_BASIC_VALID_1: &str = r#"
---
model: ollama/gpt-oss:20b
input:
  schema:
    message: string, Message
output:
  format: text
---
Basic Prompt Here: {{message}}
"#;

    const PROMPTFILE_BASIC_VALID_2: &str = r#"
---
model: ollama/gemma:27b
input:
  schema:
    message: string, Message
output:
  format: text
---
Basic Prompt Here: {{message}}
"#;

    const PROMPTFILE_INVALID_MODEL: &str = r#"
---
model: aaaa/gpt-oss:20b
input:
  schema:
    message: string, Message
output:
  format: text
---
Basic Prompt Here: {{message}}
"#;

    #[derive(Default)]
    struct TestingTextEditor {
        user_input: String
    }

    impl TestingTextEditor {
        pub fn set_user_input(&mut self, data: &str) {
            self.user_input = data.to_string().clone();
        }
    }

    impl TextEditor for TestingTextEditor {
        fn edit(&self, _: &str) -> Result<String, cmd::TextEditorError> {
            Ok(self.user_input.clone())
        }
    }

    struct TestState {
        storage: InMemoryPromptFilesStorage,
        installer: InMemoryInstaller,
        config: AppConfig,
        inp: Vec<u8>,
        out: Vec<u8>,
        editor: TestingTextEditor
    }

    fn setup(inpdata: &[u8]) -> TestState {
        TestState {
            storage: InMemoryPromptFilesStorage::default(),
            installer: InMemoryInstaller::default(),
            config: AppConfig::default(),
            inp: inpdata.to_vec(),
            out: Vec::new(),
            editor: TestingTextEditor::default()
        }
    }
    #[test]
    fn test_nochanges() {
        let mut state = setup(b"");
        let promptname = String::from("myprompt");
        state.storage.store(&promptname, PROMPTFILE_BASIC_VALID_1).unwrap();
        state.editor.set_user_input(PROMPTFILE_BASIC_VALID_1);


        cmd::edit::exec(
            &mut &state.inp[..],
            &mut std::io::stderr(),
            &mut state.storage,
            &state.editor,
            &state.config,
            EditCmd {
                force: false,
                promptname: promptname.to_string()
            }
            ).unwrap();

        let actual_promptdata = state.storage.load(&promptname).unwrap().1;

        // Provided prompt data should be stored as is
        assert_eq!(
            PROMPTFILE_BASIC_VALID_1, 
            actual_promptdata);
    }

    #[test]
    fn test_non_existent() {
        let mut state = setup(b"");
        let promptname = String::from("myprompt");

        cmd::edit::exec(
            &mut &state.inp[..],
            &mut std::io::stderr(),
            &mut state.storage,
            &state.editor,
            &state.config,
            EditCmd {
                force: false,
                promptname: promptname.to_string()
            }
            ).unwrap_err();

        state.storage.load(&promptname).unwrap_err();

    }

    #[test]
    fn test_edit_existing_successfully () {
        let mut state = setup(b"");
        let promptname = String::from("myprompt");
        state.storage.store(&promptname, PROMPTFILE_BASIC_VALID_1).unwrap();
        state.editor.set_user_input(PROMPTFILE_BASIC_VALID_2);


        cmd::edit::exec(
            &mut &state.inp[..],
            &mut std::io::stderr(),
            &mut state.storage,
            &state.editor,
            &state.config,
            EditCmd {
                force: false,
                promptname: promptname.to_string()
            }
            ).unwrap();

        let actual_promptdata = state.storage.load(&promptname).unwrap().1;

        // Provided prompt data should be stored as is
        assert_eq!(
            PROMPTFILE_BASIC_VALID_2, 
            actual_promptdata
        );
    }

    #[test]
    fn test_invalid_provider_nosave() {
        let mut state = setup(b"N\n");
        let promptname = String::from("myprompt");

        state.storage.store(&promptname, PROMPTFILE_BASIC_VALID_1).unwrap();
        state.editor.set_user_input(PROMPTFILE_INVALID_MODEL);

        cmd::edit::exec(
            &mut &state.inp[..],
            &mut std::io::stderr(),
            &mut state.storage,
            &state.editor,
            &state.config,
            EditCmd {
                force: false,
                promptname: promptname.clone()
            }
            ).unwrap();

        assert_eq!(
            state.storage.load(&promptname).unwrap().1,
            PROMPTFILE_BASIC_VALID_1
        );

    }

    #[test]
    fn test_invalid_provider_force_save_by_input() {
        let mut state = setup(b"Y\n");
        let promptname = String::from("myprompt");

        state.storage.store(&promptname, PROMPTFILE_BASIC_VALID_1).unwrap();
        state.editor.set_user_input(PROMPTFILE_INVALID_MODEL);

        cmd::edit::exec(
            &mut &state.inp[..],
            &mut std::io::stderr(),
            &mut state.storage,
            &state.editor,
            &state.config,
            EditCmd {
                force: false,
                promptname: promptname.clone()
            }
            ).unwrap();

        assert_eq!(
            state.storage.load(&promptname).unwrap().1,
            PROMPTFILE_INVALID_MODEL
        );
    }

    #[test]
    fn test_invalid_provider_force_save_by_argument() {
        let mut state = setup(b"");
        let promptname = String::from("myprompt");

        state.storage.store(&promptname, PROMPTFILE_BASIC_VALID_1).unwrap();
        state.editor.set_user_input(PROMPTFILE_INVALID_MODEL);

        cmd::edit::exec(
            &mut &state.inp[..],
            &mut std::io::stderr(),
            &mut state.storage,
            &state.editor,
            &state.config,
            EditCmd {
                force: true,
                promptname: promptname.clone()
            }
            ).unwrap();

        assert_eq!(
            state.storage.load(&promptname).unwrap().1,
            PROMPTFILE_INVALID_MODEL
        );
    }
}
