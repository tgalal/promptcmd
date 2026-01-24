use handlebars::*;
use tokio::io::AsyncReadExt;

use openssh::{KnownHosts, Session, Stdio};
pub struct RemoteExecHelper {
    pub destination: String
}

impl HelperDef for RemoteExecHelper {
    fn call<'reg: 'rc, 'rc>( &self,
            h: &Helper<'rc>,
            _: &'reg Handlebars<'reg>,
            _: &'rc Context,
            _: &mut RenderContext<'reg, 'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {

        let params = h.params();
        let cmd = params.first().ok_or(
            RenderError::from(RenderErrorReason::Other("exec binary not specified".to_string()))
        )?.render();

        let args: Vec<String> = params.iter().skip(1).map(|item| {
            if item.is_value_missing() {
                Err(RenderError::from(RenderErrorReason::Other(
                    format!("Undefined variable: {}", item.relative_path().unwrap()))))
            } else {
                Ok(item.render())
            }
        }).collect::<Result<Vec<_>, _>>()?;

        let runtime = tokio::runtime::Runtime::new()?;
        let (success, output) = runtime.block_on(async {
            // Connect to remote host
            let session = Session::connect(&self.destination, KnownHosts::Strict).await?;

            let mut child = session.command(&cmd)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            .await?;

            // Read both streams concurrently
            let stdout = child.stdout().take().unwrap();
            let stderr = child.stderr().take().unwrap();
            let mut stdout_data = Vec::new();
            let mut stderr_data = Vec::new();
            let (stdout_result, stderr_result) = tokio::join!(
                async {
                    let mut reader = stdout;
                    reader.read_to_end(&mut stdout_data).await
                },
                async {
                    let mut reader = stderr;
                    reader.read_to_end(&mut stderr_data).await
                }
            );
            stdout_result?;
            stderr_result?;

            let exit_status = child.wait().await?;
            session.close().await?;
            // Combine them
            let mut combined = stdout_data;
            combined.extend_from_slice(&stderr_data);

            Ok::<(bool, String), Box<dyn std::error::Error>>((exit_status.success(), String::from_utf8_lossy(&combined).to_string()))
        }).map_err(|err| RenderError::from(RenderErrorReason::Other(err.to_string())))?;

        if success {
            out.write(&output)?;
            Ok(())
        } else {
            let error_message = format!("Error executing command: {}, output was: {}", &cmd, &output);
            Err(RenderError::from(RenderErrorReason::Other(error_message)))
        }
    }
}
