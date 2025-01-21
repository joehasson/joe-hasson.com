use anyhow;
use tera::{Context, Tera};

#[derive(Debug, Clone)]
pub struct SsrCommon {
    tera: Tera,
    base_context: Context,
}

impl SsrCommon {
    pub fn load() -> Result<Self, anyhow::Error> {
        let tera = Tera::new("templates/*")?;
        let raw_css = std::fs::read_to_string("build/css/bundle.css")?;
        let mut base_context = Context::new();
        base_context.insert("css", &raw_css);
        Ok(Self { tera, base_context })
    }

    pub fn render(&self, template: &str) -> Result<String, tera::Error> {
        self.tera.render(template, &self.base_context)
    }

    pub fn with_context(mut self, key: &str, val: &str) -> Self {
        self.base_context.insert(key, val);
        self
    }
}
