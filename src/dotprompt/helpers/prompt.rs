use std::{sync::Arc};

use handlebars::*;

use crate::executor::{ExecutionOutput, Executor, ExecutorErorr, PromptInputs};
pub struct PromptHelper {
    pub executor: Arc<Executor>,
    pub dry: bool,
}

impl HelperDef for PromptHelper {
    fn call<'reg: 'rc, 'rc>(
            &self,
            h: &Helper<'rc>,
            _: &'reg Handlebars<'reg>,
            _: &'rc Context,
            _: &mut RenderContext<'reg, 'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {

        let promptname = h.params().first().ok_or(
            RenderError::from(RenderErrorReason::Other("prompt name not specified".to_string()))
        )?.render();

        let params = h.hash();

        let mut inputs = PromptInputs::new();

        for (k, v) in params {
           inputs.insert(k.to_string(), v.value().clone());
        }


        let result = self.executor.clone().execute(&promptname, None, None, inputs, self.dry).map_err(|err: ExecutorErorr| {
            RenderError::from(RenderErrorReason::Other(err.to_string()))
        })?;

        match result {
            ExecutionOutput::StreamingOutput(mut stream) => {
                let output = stream.sync_collect().map_err( |err|
                    RenderError::from(RenderErrorReason::Other(err.to_string()))
                )?;
                out.write(output.as_str())?;
            },
            ExecutionOutput::StructuredStreamingOutput(mut stream) => {
                let output = stream.sync_collect().map_err( |err|
                    RenderError::from(RenderErrorReason::Other(err.to_string()))
                )?;
                out.write(output.as_str())?;
            }
            ExecutionOutput::ImmediateOutput(output) => {
                out.write(&output)?;
            },
            ExecutionOutput::Cached(output) => {
                out.write(&output)?;
            },
            ExecutionOutput::DryRun => {}
        };

        Ok(())
    }
}
