pub mod edit;
pub mod enable;
pub mod disable;
pub mod create;
pub mod list;
pub mod cat;
pub mod run;
pub mod import;
pub mod stats;
pub mod resolve;
pub mod config;
pub mod render;
pub mod ssh;

mod templates;

use clap::value_parser;
use clap::Arg;
use clap::ArgGroup;
use clap::Command;
use thiserror::Error;

use ::edit::Builder;
use ::edit::edit_with_builder;

#[derive(Error, Debug)]
pub enum TextEditorError {
    #[error("Editor Error")]
    IoError(#[from] std::io::Error)
}

pub enum TextEditorFileType {
    Toml,
    Dotprompt
}

pub trait TextEditor {
    fn edit(&self, input: &str, eftype: TextEditorFileType) -> Result<String, TextEditorError>;
}

pub struct BasicTextEditor;

impl TextEditor for BasicTextEditor {
    fn edit(&self, input: &str, eftype: TextEditorFileType) -> Result<String, TextEditorError> {

        let mut b = Builder::new();

        match eftype {
            TextEditorFileType::Toml => b.suffix(".toml"),
            TextEditorFileType::Dotprompt => b.suffix(".prompt"),
        };

        let result = edit_with_builder(input, &b);

        result.map_err(TextEditorError::IoError)
    }
}

pub fn command_add_remote_options(mut command: Command) -> Command {
    command = command.next_help_heading("Remote Options")
        .arg(
            Arg::new("remote_dest")
            .long("remote-dest")
            .help("Execute commands on a remote SSH destination")
        )
        .arg(
            Arg::new("remote_port")
            .long("remote-port")
            .value_parser(value_parser!(u32))
            .help("Port to use with remote destination")
        );
    command
}

pub fn command_add_general_options(mut command: Command) -> Command {
    command = command.next_help_heading("General Options");
    command = command.arg(Arg::new("dry")
            .long("dry")
            .help("Dry run")
            .action(clap::ArgAction::SetTrue)
            .required(false)
        )
        .arg(Arg::new("render")
            .long("render")
            .short('r')
            .help("Render only mode")
            .action(clap::ArgAction::SetTrue)
            .required(false)
        )
        .arg(
            Arg::new("help")
            .long("help")
            .short('h')
            .action(clap::ArgAction::Help)
            .help("Print help")
        );
    command
}

pub fn command_add_configuration_options(mut command: Command) -> Command {
    command = command.next_help_heading("Optional Configuration Overrides")
        .arg(Arg::new("model")
            .long("config-model")
            .short('m')
        )
        .arg(Arg::new("stream")
            .long("config-stream")
            .action(clap::ArgAction::SetTrue)
        )
        .arg(Arg::new("nostream")
            .long("config-no-stream")
            .action(clap::ArgAction::SetTrue)
        )
        .group(ArgGroup::new("streamgroup").args(["stream", "nostream"]))
        .arg(Arg::new("cache_ttl")
            .long("config-cache-ttl")
            .value_parser(value_parser!(u32))
        )
        .arg(Arg::new("temperature")
            .long("config-temperature")
            .alias("config-temp")
            .value_parser(value_parser!(f32))
        )
        .arg(Arg::new("max_tokens")
            .long("config-max-tokens")
            .value_parser(value_parser!(u32))
        )
        .arg(Arg::new("system")
            .long("config-system")
        )
        ;
    command
}
