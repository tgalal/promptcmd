use std::collections::HashMap;

pub trait Render<T, E> {
    fn render(&self, t: &T, extra: &HashMap<String, String>) -> Result<String, E>;
}


