use handlebars::*;

use crate::{dotprompt::helpers::{handle_multiplexed_session,
    tail::TailHelperArguments}, executor::RemoteExecContext};

pub struct RemoteTailHelper {
    pub context: RemoteExecContext
}

impl RemoteTailHelper {
    async fn async_call<'reg: 'rc, 'rc>( &self,
            h: &Helper<'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {

        let helper_args = TailHelperArguments::try_from(h)?;

        let cmd = "tail";

        let args: Vec<String> = vec![
            String::from("-n"),
            helper_args.lines.to_string(),
            helper_args.filename
        ];

        match &self.context {
            RemoteExecContext::MultiplexedSession(session_info) => handle_multiplexed_session(
                session_info, cmd, &args, out).await
        }
    }
}

impl HelperDef for RemoteTailHelper {
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
