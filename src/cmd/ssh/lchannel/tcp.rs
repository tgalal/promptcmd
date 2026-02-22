use std::sync::Arc;
use tokio::net::{TcpListener};

use crate::cmd::ssh::lchannel::LChannel;
use crate::executor::Executor;
use super::stream_common;
use super::ChannelError;
use async_trait::async_trait;


pub struct TcpChannel {
    pub executor: Arc<Executor>,
    pub port: u32,
    pub session_pwd: String
}

#[async_trait]
impl LChannel for  TcpChannel {
    async fn run(&self) -> Result<(), ChannelError> {

        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;

        loop {
            let (tcpstream, _) =listener.accept().await?;
            let executor = self.executor.clone();
            let (reader, writer) = tcpstream.into_split();

            let session_pwd = self.session_pwd.clone();

            tokio::spawn(async move {
                stream_common::handle_stream(executor, reader, writer, &session_pwd).await
            });
        }
    }
}
