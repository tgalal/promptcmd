use std::{path::PathBuf, sync::Arc};
use thiserror::Error;

use crate::{cmd::ssh::{lchannel::{self, LChannel}},
    executor::{ExecContext, Executor, RemoteExecContext}};

use crate::remote_shell::{Channel, ShellError};

pub struct BootstrapData {
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
    channel: &Channel,
    session_pwd: &str
) -> Result<BootstrapData, BootstrapError> {
    match (channel,  &executor.exec_context) {
        (Channel::Nc(local_port, remote_port), _)  => {

            Ok(BootstrapData {
                forwards: vec![ForwardingConfiguration {remote: remote_port.to_string(), local: format!("localhost:{local_port}")}],
                lchannel: Box::new(lchannel::tcp::TcpChannel {
                    executor,
                    port: *local_port,
                    session_pwd: session_pwd.to_string()
                })
            })
        },
        (Channel::Socat(local_socket, remote_socket), _) => {
            Ok(BootstrapData {
                forwards: vec![ForwardingConfiguration {remote: remote_socket.to_string(), local: local_socket.to_string()}],
                lchannel: Box::new(lchannel::usock::USocketChannel {
                    executor,
                    path: PathBuf::from(local_socket),
                    session_pwd: session_pwd.to_string()
                })
            })
        },
        (Channel::BashTcp(local_port, remote_port), _) => {
            Ok(BootstrapData {
                forwards: vec![ForwardingConfiguration {remote: remote_port.to_string(), local: format!("localhost:{local_port}")}],
                lchannel: Box::new(lchannel::tcp::TcpChannel {
                    executor,
                    port: *local_port,
                    session_pwd: session_pwd.to_string()
                })
            })
        },
        (Channel::Fifo(workdir), ExecContext::Remote(RemoteExecContext::MultiplexedSession(session))) => {
            Ok(BootstrapData {
                forwards: vec![],
                lchannel: Box::new(lchannel::multissh::MultiSshChannel {
                    session: session.clone(),
                    executor,
                    rendezvous_path: format!("{workdir}/rendezvous"),
                    session_pwd: session_pwd.to_string()
                })
            })
        },
        (Channel::FifoSingle(workdir), ExecContext::Remote(RemoteExecContext::MultiplexedSession(session))) => {
            Ok(BootstrapData {
                forwards: vec![],
                lchannel: Box::new(lchannel::ssh::SshChannel {
                    session: session.clone(),
                    executor,
                    send_path: format!("{workdir}/send"),
                    recv_path: format!("{workdir}/recv"),
                    session_pwd: session_pwd.to_string()
                })
            })
        },
        (_, ExecContext::Local) => {
            Err(BootstrapError::UnsupportedExecContext("Local".to_string()))
        },
        (_ , ExecContext::Remote(RemoteExecContext::Other)) => {
            todo!()
        }
    }
}
