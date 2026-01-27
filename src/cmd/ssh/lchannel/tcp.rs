use std::sync::Arc;
use tokio::net::{TcpListener};

use crate::executor::Executor;
use super::stream_common;
use super::ChannelError;


pub struct TcpChannel {
    pub executor: Arc<Executor>,
    pub port: u32
}

impl TcpChannel {
    pub async fn run(&self) -> Result<(), ChannelError> {

        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;

        loop {
            let (tcpstream, _) =listener.accept().await?;
            let executor = self.executor.clone();

            tokio::spawn(async {
                stream_common::handle_stream(executor, tcpstream).await
            });
        }
    }
}
