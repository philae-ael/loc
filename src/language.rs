use crate::line_kind::{Generic, GenericWithComment, LineKindEstimator, MultilineCommentAware};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Language {
    Rust,
    C,
    Js,
    Json,
    Python,
    Generic,
    Toml,
    Go,
    Yaml,
    Markdown,
    VueJs,
    Css,
    Csv,
    Dockerfile,
    Shader,
    CMake,
    Makefile,
    Asset,
    Tex,
    Liquid,
    Ruby,
    Html,
    Shell,
    Txt,
    Slang,
    Lockfile,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let this = match self {
            Language::Rust => "Rust",
            Language::C => "C/C++",
            Language::Generic => "Other",
            Language::Js => "Javascript",
            Language::Json => "JSON",
            Language::Python => "Python",
            Language::Toml => "TOML",
            Language::Go => "GO",
            Language::Yaml => "YAML",
            Language::Markdown => "Markdown",
            Language::VueJs => "VueJs",
            Language::Css => "CSS",
            Language::Html => "HTML",
            Language::Csv => "CSV",
            Language::Dockerfile => "Dockerfile",
            Language::Shader => "Shader",
            Language::CMake => "CMake",
            Language::Asset => "Asset",
            Language::Makefile => "Makefile",
            Language::Tex => "Tex/Latex",
            Language::Txt => "Text",
            Language::Shell => "Shell",
            Language::Ruby => "Ruby",
            Language::Liquid => "Liquid",
            Language::Lockfile => "Lockfile",
            Language::Slang => "Slang",
        };
        write!(f, "{this}")
    }
}

pub fn make_line_kind_estimator(language: Language) -> Option<Box<dyn LineKindEstimator + Send>> {
    match language {
        Language::Rust => Some(Box::new(GenericWithComment::new("//"))),
        Language::VueJs
        | Language::Slang
        | Language::C
        | Language::Js
        | Language::Go
        | Language::Shader => Some(Box::new(MultilineCommentAware::new("//", ["/*", "*/"]))),
        Language::Python | Language::Toml => Some(Box::new(GenericWithComment::new("#"))),
        Language::Tex => Some(Box::new(GenericWithComment::new("%"))),
        Language::Markdown
        | Language::Makefile
        | Language::Css
        | Language::Yaml
        | Language::Html
        | Language::Csv
        | Language::Liquid
        | Language::Dockerfile
        | Language::Generic
        | Language::CMake
        | Language::Ruby
        | Language::Txt
        | Language::Shell
        | Language::Json => Some(Box::new(Generic)),
        Language::Lockfile | Language::Asset => None,
    }
}
