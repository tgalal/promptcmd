use std::{path::PathBuf, sync::Arc};
use thiserror::Error;

use crate::{cmd::ssh::{lchannel::{self, LChannel}, shell::{Channel, Shell, ShellError}}, executor::Executor};

pub struct BootstrapData {
    pub script: String,
    pub forwards: Vec<ForwardingConfiguration>,
    pub lchannel: Box<dyn LChannel>
}

pub struct ForwardingConfiguration {
    pub local: String,
    pub remote: String
}

#[derive(Error, Debug)]
pub enum BootstrapError {
    #[error("{0}")]
    ShellError(#[from] ShellError)
}

pub fn setup(executor: Arc<Executor>, promptnames: &[String], shell: Shell,
    channel: Channel, remote_cmd: Option<&[&str]>) -> Result<BootstrapData, BootstrapError> {
    let bootstrap_script = shell.build(&channel, promptnames, remote_cmd)?;

    let res = match channel {
        Channel::Nc(local_port, remote_port) => {

            BootstrapData {
                script: bootstrap_script,
                forwards: vec![ForwardingConfiguration {remote: remote_port.to_string(), local: local_port.to_string()}],
                lchannel: Box::new(lchannel::tcp::TcpChannel {
                    executor,
                    port: local_port
                })
            }
        },
        Channel::Socat(local_socket, remote_socket) => {
            BootstrapData {
                script: bootstrap_script,
                forwards: vec![ForwardingConfiguration {remote: remote_socket.to_string(), local: local_socket.to_string()}],
                lchannel: Box::new(lchannel::usock::USocketChannel {
                    executor,
                    path: PathBuf::from(local_socket)
                })
            }
        },
        Channel::BashTcp(local_port, remote_port) => {
            BootstrapData {
                script: bootstrap_script,
                forwards: vec![ForwardingConfiguration {remote: remote_port.to_string(), local: local_port.to_string()}],
                lchannel: Box::new(lchannel::tcp::TcpChannel {
                    executor,
                    port: local_port
                })
            }
        },
    };
    Ok(res)
}
