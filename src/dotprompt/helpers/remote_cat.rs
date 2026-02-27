use handlebars::*;

use crate::{dotprompt::helpers::{cat::CatHelperArguments, handle_multiplexed_session}, executor::RemoteExecContext};
pub struct RemoteCatHelper {
    pub context: RemoteExecContext
}

// {{cat filename}}
impl RemoteCatHelper {
    async fn async_call<'reg: 'rc, 'rc>( &self,
            h: &Helper<'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {

        let helper_args = CatHelperArguments::try_from(h)?;

        let cmd = "cat";
        let args: Vec<String> = vec![helper_args.filename];

        match &self.context {
            RemoteExecContext::MultiplexedSession(session_info) => handle_multiplexed_session(
                session_info, cmd, &args, out).await
        }
    }
}

impl HelperDef for RemoteCatHelper {
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
