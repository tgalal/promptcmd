use handlebars::*;

use crate::{dotprompt::helpers, executor::RemoteExecContext};

pub struct RemoteHeadHelper {
    pub context: RemoteExecContext
}

impl RemoteHeadHelper {
    async fn async_call<'reg: 'rc, 'rc>( &self,
            h: &Helper<'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {

        let helper_args = helpers::head::HeadHelperArguments::try_from(h)?;

        let args: Vec<String> = vec![
            String::from("-n"),
            helper_args.lines.to_string(),
            helper_args.filename
        ];

        match &self.context {
            #[cfg(not(target_os="windows"))]
            RemoteExecContext::MultiplexedSession(session_info) => helpers::handle_multiplexed_session(
                session_info, "head", &args, out).await,
            RemoteExecContext::Other => todo!()
        }
    }
}

impl HelperDef for RemoteHeadHelper {
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
