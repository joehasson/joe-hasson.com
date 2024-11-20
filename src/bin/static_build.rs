use tera::{Tera, Context};
use anyhow::Result;
use lightningcss::stylesheet::{StyleSheet, ParserOptions};

fn main() -> Result<()> {
    // Work in the project root
    let executable_path = std::env::current_exe()?;
    let project_root = executable_path
        .parent().expect("Failed to find project root")
        .parent().expect("Failed to find project root")
        .parent().expect("Failed to find project root");
    std::env::set_current_dir(project_root)?;

    // perform CSS bundling
    let mut bundled_css = String::new();
    for dir_entry in std::fs::read_dir("styles/")? {
        let dir_entry = dir_entry?;
        let path = dir_entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("css") {
            let raw = std::fs::read_to_string(path)?;
            let mut stylesheet = StyleSheet::parse(&raw, ParserOptions::default()).unwrap();
            stylesheet.minify(Default::default())?;
            let code = stylesheet.to_css(Default::default())?.code;
            bundled_css.push_str(&code);
        }
    }

    // Set up tera for template rendering
    let tera = Tera::new("templates/*")?;
    let mut context = Context::new();
    context.insert("css", &bundled_css);

    // Render templates
    std::fs::create_dir_all("build")?;
    for fname in tera.get_template_names() {
        let rendered = tera.render(fname, &context)?;
        std::fs::write(format!("build/{}", fname), rendered)?;
    }

    Ok(())
}
