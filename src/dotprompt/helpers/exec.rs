use handlebars::*;

use std::{io::Read, process::Command};
pub struct ExecHelper;

impl HelperDef for ExecHelper {
    fn call<'reg: 'rc, 'rc>(
            &self,
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

        let (mut reader, writer) = std::io::pipe()?;

        let child =  {
            Command::new(&cmd)
            .args(&args)
            .stdout(writer.try_clone()?)
            .stderr(writer)
            .output()?
        };

        let mut output = String::new();
        reader.read_to_string(&mut output)?;

        if child.status.success() {
            out.write(&output)?;
            Ok(())
        } else {
            let error_message = format!("Error executing command: {}, output was: {}", &cmd, &output);
            Err(RenderError::from(RenderErrorReason::Other(error_message)))
        }
    }
}
