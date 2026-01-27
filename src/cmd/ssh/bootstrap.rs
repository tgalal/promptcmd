use std::{path::PathBuf, sync::Arc};

use crate::{cmd::ssh::{lchannel::{self, LChannel}, shell::{Channel, Shell}}, executor::Executor};

pub struct BootstrapData {
    pub script: String,
    pub forwards: Vec<String>,
    pub lchannel: Box<dyn LChannel>
}

pub fn setup(executor: Arc<Executor>, promptnames: &[String], shell: Shell, channel: Channel) -> BootstrapData {
    let bootstrap_script = shell.build(&channel, promptnames);

    match channel {
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

    }
}
