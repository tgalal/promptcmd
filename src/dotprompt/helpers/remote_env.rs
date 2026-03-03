use handlebars::*;

use crate::{dotprompt::helpers, executor::RemoteExecContext};
pub struct RemoteEnvHelper {
    pub context: RemoteExecContext
}

impl RemoteEnvHelper {
    async fn async_call<'reg: 'rc, 'rc>( &self,
            h: &Helper<'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {

        let helper_args = helpers::env::EnvHelperArguments::try_from(h)?;

        let args: Vec<String> = vec![format!(" ${}", helper_args.name)];

        match &self.context {
            #[cfg(not(target_os="windows"))]
            RemoteExecContext::MultiplexedSession(session_info) => helpers::handle_multiplexed_session(
                session_info, "echo", &args, out, helper_args.default).await,
            RemoteExecContext::Other => todo!()
        }
    }
}

impl HelperDef for RemoteEnvHelper {
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
