use handlebars::*;
use rev_lines::RevLines;
use std::fs::File;

pub struct TailHelper;

pub struct TailHelperArguments {
    pub filename: String,
    pub lines: usize
}

impl<'rc> TryFrom<&Helper<'rc>> for TailHelperArguments {
    type Error = RenderError;

    fn try_from(h: &Helper) -> Result<Self, Self::Error> {
        let params = h.params();
        let args = h.hash();

        let filename = params.first().ok_or(
            RenderError::from(RenderErrorReason::Other("filename not specified".to_string()))
        )?.render();

        let nlines = args.get("lines")
            .map(|lines| lines.render()).unwrap_or(String::from("10"));

        let nlines_parsed = nlines
            .parse::<usize>().map_err(|_|
                RenderError::from(RenderErrorReason::Other(format!("Expected a positive integer for lines, found {nlines}")))
            )?;

        Ok(Self {
            filename,
            lines: nlines_parsed
        })
    }
}

impl HelperDef for TailHelper {

    fn call<'reg: 'rc, 'rc>(
            &self,
            h: &Helper<'rc>,
            _: &'reg Handlebars<'reg>,
            _: &'rc Context,
            _: &mut RenderContext<'reg, 'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {

        let args: TailHelperArguments = h.try_into()?;
        let filename = &args.filename;
        let lines = args.lines;

        let file = File::open(filename)?;
        let rev_lines = RevLines::new(file);

        let last_lines = rev_lines
            .take(lines)
            .collect::<Result<Vec<_>, _>>().map_err(|e|
                RenderError::from(RenderErrorReason::Other(format!("Error reading line: {e}")))
            )?
            .into_iter()
            .rev() // restore original order
            .collect::<Vec<_>>()
            .join("\n");

        out.write(&last_lines)?;

        Ok(())
        }
}
