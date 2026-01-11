use std::collections::HashMap;

use handlebars::Handlebars;

use crate::{dotprompt::{renderers::{Render, RenderError}, DotPrompt}, executor::PromptInputs};

impl Render<PromptInputs> for DotPrompt {
    fn render(&self,
            kv: PromptInputs,
            helpers: HashMap<&str, Box<dyn handlebars::HelperDef + Send + Sync>>
        ) -> Result<String,RenderError> {

        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);

        for (k, v) in helpers {
            hbs.register_helper(k, v);
        }

        // hbs.register_helper("exec", Box::new(ExecHelper));
        // hbs.register_helper("prompt", Box::new());

        let template_name = &self.name;
        hbs.register_template_string(template_name, &self.template)?;

        let output = hbs.render(template_name, &kv.map)?;

        Ok(output)
    }

}

