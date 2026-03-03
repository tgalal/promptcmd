use std::env;

use handlebars::*;

pub struct EnvHelper;

pub struct EnvHelperArguments {
    pub name: String,
    pub default: Option<String>
}

impl<'rc> TryFrom<&Helper<'rc>> for EnvHelperArguments {
    type Error = RenderError;

    fn try_from(h: &Helper) -> Result<Self, Self::Error> {
        let params = h.params();

        let name = params.first().ok_or(
            RenderError::from(RenderErrorReason::Other("env field name not specified".to_string()))
        )?.render();
        let default = params.get(1)
            .map(|val| Some(val.render()))
            .unwrap_or(Some(String::from("")));

        Ok(Self {
            name,
            default
        })
    }
}

impl HelperDef for EnvHelper {
    fn call<'reg: 'rc, 'rc>(
            &self,
            h: &Helper<'rc>,
            _: &'reg Handlebars<'reg>,
            _: &'rc Context,
            _: &mut RenderContext<'reg, 'rc>,
            out: &mut dyn Output,
        ) -> HelperResult {

        let helper_args: EnvHelperArguments = h.try_into()?;

        let value = env::var(helper_args.name).unwrap_or(
            helper_args.default.unwrap_or(String::from(""))
        );

        out.write(&value)?;

        Ok(())
    }
}
