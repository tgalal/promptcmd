use crate::{config::appconfig::BashMethod};
use thiserror::Error;

pub mod zsh;
pub mod sh;
pub mod shell_stage3;
pub mod fish;
pub mod bashrc;
pub mod bashposix;
pub mod bashexports;

#[derive(Error, Debug)]
pub enum ShellError {
    #[error("{0}")]
    UnsupportedMode(String)
}

#[derive(Debug)]
pub enum Shell {
    Bash(BashMethod, String),
    Zsh(String),
    Sh(String),
    Ash(String),
    Dash(String),
    Fish(String),
    Auto(String)
}

#[derive(Debug)]
pub enum Channel {
    // local_port, remote_port
    Nc(u32, u32),
    // local_socket, remote_socket
    Socat(String, String),
    // local_port, remote_port
    BashTcp(u32, u32),
    // workdir
    Fifo(String),
    // workdir
    FifoSingle(String),
    // remote_port, remote_file
    // Auto(u32, String)
}

pub trait Stage1 {
    fn stage1(&self, stage2: &str) -> String;
}

pub trait Stage2 {
    fn stage2(&self, dispatcher_name: &str, promptnames: &[String],
        channel: &Channel, shell: &Shell, bash_method: &BashMethod, remote_cmd: Option<&str>)-> String;
}

pub trait Stage3 {
    fn stage3(&self, functions_file: &str, prompts_names: &[String], remote_cmd: Option<&str>) -> String;
    fn create_prompt_functions(&self, dispatcher_name: &str, prompts_names: &[String]) -> String;
}
