use std::{io::{BufRead, BufReader, Read}, sync::Mutex};
use std::io::{Write};

use handlebars::*;

pub struct AskHelper<R: Read> {
    pub promptname: String,
    pub inp: Mutex<BufReader<R>>
}

impl<R:Read> HelperDef for AskHelper<R> {
    fn call<'reg: 'rc, 'rc>(
            &self,
            h: &Helper<'rc>,
            _: &'reg Handlebars<'reg>,
            _: &'rc Context,
            _: &mut RenderContext<'reg, 'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {
        let question = h.params().first().ok_or(
            RenderError::from(RenderErrorReason::Other("question not specified".to_string()))
        )?.render();

        let mut buffer = String::new();

        print!("{}> {}", &self.promptname, &question);
        std::io::stdout().flush()?;
        let mut inp = self.inp.lock().unwrap();
        inp.read_line(&mut buffer)?;


        out.write(buffer.as_str().trim())?;

        Ok(())
    }
}
