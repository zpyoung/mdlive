use minijinja::Environment;
use std::sync::OnceLock;

pub(crate) const TEMPLATE_NAME: &str = "main.html";
pub(crate) static TEMPLATE_ENV: OnceLock<Environment<'static>> = OnceLock::new();
pub(crate) const MERMAID_JS: &str = include_str!("../static/js/mermaid.min.js");
pub(crate) const HIGHLIGHT_JS: &str = include_str!("../static/js/highlight.min.js");
pub(crate) const MARKED_JS: &str = include_str!("../static/js/marked.min.js");
pub(crate) const MD_ICON_PNG: &[u8] = include_bytes!("../static/img/md.png");
pub(crate) const MDLIVE_LOGO_PNG: &[u8] = include_bytes!("../static/img/mdlive.png");
pub(crate) const STATIC_ETAG: &str = concat!("\"", env!("CARGO_PKG_VERSION"), "\"");

pub(crate) fn template_env() -> &'static Environment<'static> {
    TEMPLATE_ENV.get_or_init(|| {
        let mut env = Environment::new();
        minijinja_embed::load_templates!(&mut env);
        env
    })
}
