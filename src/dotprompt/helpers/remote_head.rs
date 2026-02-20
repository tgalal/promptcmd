
use handlebars::*;

use crate::{dotprompt::helpers::{handle_destination, handle_multiplexed_session,
    head::HeadHelperArguments}, executor::RemoteExecContext};

pub struct RemoteHeadHelper {
    pub context: RemoteExecContext
}

impl RemoteHeadHelper {
    async fn async_call<'reg: 'rc, 'rc>( &self,
            h: &Helper<'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {

        let helper_args = HeadHelperArguments::try_from(h)?;

        let cmd = "head";

        let args: Vec<String> = vec![
            String::from("-n"),
            helper_args.lines.to_string(),
            helper_args.filename
        ];

        match &self.context {
            RemoteExecContext::Destination(destination) => handle_destination(destination.as_str(), cmd, &args, out).await,
            RemoteExecContext::MultiplexedSession(session_info) => handle_multiplexed_session(
                session_info, cmd, &args, out).await
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
