use std::fs;

use handlebars::*;

pub struct CatHelper;

pub struct CatHelperArguments {
    pub filename: String
}

impl<'rc> TryFrom<&Helper<'rc>> for CatHelperArguments {
    type Error = RenderError;

    fn try_from(h: &Helper) -> Result<Self, Self::Error> {
        let params = h.params();

        let filename = params.first().ok_or(
            RenderError::from(RenderErrorReason::Other("filename not specified".to_string()))
        )?.render();

        Ok(Self {
            filename,
        })
    }
}

impl HelperDef for CatHelper {
    fn call<'reg: 'rc, 'rc>(
            &self,
            h: &Helper<'rc>,
            _: &'reg Handlebars<'reg>,
            _: &'rc Context,
            _: &mut RenderContext<'reg, 'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {

        let helper_args: CatHelperArguments = h.try_into()?;

        let content = fs::read_to_string(helper_args.filename)?;

        out.write(&content)?;

        Ok(())
    }
}
