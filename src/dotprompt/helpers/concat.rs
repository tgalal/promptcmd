use handlebars::*;

pub struct ConcatHelper;

impl HelperDef for ConcatHelper {
    fn call<'reg: 'rc, 'rc>(
            &self,
            h: &Helper<'rc>,
            _: &'reg Handlebars<'reg>,
            _: &'rc Context,
            _: &mut RenderContext<'reg, 'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {
        let params = h.params();

        let concatenated = params.iter().map(|item| item.render()).collect::<Vec<String>>().join("");
        out.write(&concatenated)?;

        Ok(())
    }
}
