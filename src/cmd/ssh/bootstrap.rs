use std::{path::PathBuf, sync::Arc};
use thiserror::Error;

use crate::{cmd::ssh::{lchannel::{self, LChannel}, shell::{Channel, Shell, ShellError}}, executor::Executor};

pub struct BootstrapData {
    pub script: String,
    pub forwards: Vec<String>,
    pub lchannel: Box<dyn LChannel>
}

#[derive(Error, Debug)]
pub enum BootstrapError {
    #[error("{0}")]
    ShellError(#[from] ShellError)
}

pub fn setup(executor: Arc<Executor>, promptnames: &[String], shell: Shell, channel: Channel) -> Result<BootstrapData, BootstrapError> {
    let bootstrap_script = shell.build(&channel, promptnames)?;

    let res = match channel {
        Channel::Nc(local_port, remote_port) => {
            BootstrapData {
                script: bootstrap_script,
                forwards: vec![format!("{remote_port}:localhost:{local_port}")],
                lchannel: Box::new(lchannel::tcp::TcpChannel {
                    executor,
                    port: local_port
                })
            }
        },
        Channel::Socat(local_socket, remote_socket) => {
            BootstrapData {
                script: bootstrap_script,
                forwards: vec![format!("{remote_socket}:{local_socket}")],
                lchannel: Box::new(lchannel::usock::USocketChannel {
                    executor,
                    path: PathBuf::from(local_socket)
                })
            }
        },
        Channel::BashTcp(local_port, remote_port) => {
            BootstrapData {
                script: bootstrap_script,
                forwards: vec![format!("{remote_port}:localhost:{local_port}")],
                lchannel: Box::new(lchannel::tcp::TcpChannel {
                    executor,
                    port: local_port
                })
            }
        },
    };
    Ok(res)
}
