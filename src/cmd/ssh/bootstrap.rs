use std::{path::PathBuf, sync::Arc};
use thiserror::Error;

use crate::{cmd::ssh::{lchannel::{self, LChannel}, shell::{Channel, Shell, ShellError}}, executor::{ExecContext, Executor, RemoteExecContext}};

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
    ShellError(#[from] ShellError),
    #[error("Unsupported Channel: {0}")]
    UnsupportedChannel(String),
    #[error("Unsupported Execution Context: {0}")]
    UnsupportedExecContext(String),
}

pub fn setup(
    executor: Arc<Executor>,
    promptnames: &[String],
    shell: Shell,
    channel: Channel,
    remote_cmd: Option<&[&str]>
) -> Result<BootstrapData, BootstrapError> {
    let bootstrap_script = shell.build(&channel, promptnames, remote_cmd)?;

    match (channel,  &executor.exec_context) {
        (Channel::Nc(local_port, remote_port), _)  => {

            Ok(BootstrapData {
                script: bootstrap_script,
                forwards: vec![ForwardingConfiguration {remote: remote_port.to_string(), local: format!("localhost:{local_port}")}],
                lchannel: Box::new(lchannel::tcp::TcpChannel {
                    executor,
                    port: local_port
                })
            })
        },
        (Channel::Socat(local_socket, remote_socket), _) => {
            Ok(BootstrapData {
                script: bootstrap_script,
                forwards: vec![ForwardingConfiguration {remote: remote_socket.to_string(), local: local_socket.to_string()}],
                lchannel: Box::new(lchannel::usock::USocketChannel {
                    executor,
                    path: PathBuf::from(local_socket)
                })
            })
        },
        (Channel::BashTcp(local_port, remote_port), _) => {
            Ok(BootstrapData {
                script: bootstrap_script,
                forwards: vec![ForwardingConfiguration {remote: remote_port.to_string(), local: format!("localhost:{local_port}")}],
                lchannel: Box::new(lchannel::tcp::TcpChannel {
                    executor,
                    port: local_port
                })
            })
        },
        (Channel::Fifo(workdir), ExecContext::Remote(RemoteExecContext::MultiplexedSession(session))) => {
            Ok(BootstrapData {
                script: bootstrap_script,
                forwards: vec![],
                lchannel: Box::new(lchannel::ssh::SshChannel {
                    session: session.clone(),
                    executor,
                    send_path: format!("{workdir}/send"),
                    recv_path: format!("{workdir}/recv"),
                })
            })
        },
        (_, ExecContext::Remote(RemoteExecContext::Destination(_))) => {
            Err(BootstrapError::UnsupportedExecContext("Destination".to_string()))
        },
        (_, ExecContext::Local) => {
            Err(BootstrapError::UnsupportedExecContext("Local".to_string()))
        },
    }
}
