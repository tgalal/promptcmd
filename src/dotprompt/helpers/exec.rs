use handlebars::*;

use crate::dotprompt::helpers::handle_local_cmd;
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

        handle_local_cmd(&cmd, &args, out)
    }
}
