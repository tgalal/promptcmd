use std::{io::{BufReader, Read}, sync::Mutex};
use log::debug;

use handlebars::*;

pub struct StdinHelper<R: Read> {
    pub inp: Mutex<BufReader<R>>,
    pub preset_stdin: Option<String>
}

impl<R:Read> HelperDef for StdinHelper<R> {
    fn call<'reg: 'rc, 'rc>(
            &self,
            _: &Helper<'rc>,
            _: &'reg Handlebars<'reg>,
            _: &'rc Context,
            _: &mut RenderContext<'reg, 'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {

        if let Some(preset_stdin) = self.preset_stdin.as_ref() {
            out.write(preset_stdin.as_str())?;
        } else {
            let mut buffer = String::new();
            let mut inp = self.inp.lock().unwrap();

            debug!("Capturing stdin");
            inp.read_to_string(&mut buffer)
                .map_err(|err| {
                    RenderError::from(RenderErrorReason::Other(err.to_string()))
                })?;
            debug!("Done");
            out.write(buffer.as_str())?;
        }

        Ok(())
    }
}
