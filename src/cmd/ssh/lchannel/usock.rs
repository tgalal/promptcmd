use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UnixListener;
use crate::executor::Executor;
use super::stream_common;

use super::ChannelError;


pub struct USocketChannel {
    pub executor: Arc<Executor>,
    pub path: PathBuf,
}

impl USocketChannel {
    pub async fn run(&self) -> Result<(), ChannelError> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }

        let listener = UnixListener::bind(&self.path)?;

        loop {
            let (stream, _) = listener.accept().await?;
            let executor = self.executor.clone();

            tokio::spawn(async {
                stream_common::handle_stream(executor, stream).await
            });
        }
    }
}
