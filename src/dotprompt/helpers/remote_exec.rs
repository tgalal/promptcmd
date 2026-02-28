use handlebars::*;

use crate::{dotprompt::helpers};
use crate::{executor::RemoteExecContext};
pub struct RemoteExecHelper {
    pub context: RemoteExecContext
}

impl RemoteExecHelper {
    async fn async_call<'reg: 'rc, 'rc>( &self,
            h: &Helper<'rc>,
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

        match &self.context {
            #[cfg(not(target_os="windows"))]
            RemoteExecContext::MultiplexedSession(session_info) => helpers::handle_multiplexed_session(
                session_info, &cmd, &args, out).await,
            RemoteExecContext::Other => todo!()
        }
    }
}

impl HelperDef for RemoteExecHelper {
    fn call<'reg: 'rc, 'rc>( &self,
            h: &Helper<'rc>,
            _: &'reg Handlebars<'reg>,
            _: &'rc Context,
            _: &mut RenderContext<'reg, 'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.async_call(h, out).await
            })
        })
    }
}
