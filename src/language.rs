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
    Scss,
    Csv,
    Dockerfile,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let this = match self {
            Language::Rust => "Rust",
            Language::C => "C",
            Language::Generic => "Other",
            Language::Js => "Javascript",
            Language::Json => "JSON",
            Language::Python => "Python",
            Language::Toml => "TOML",
            Language::Go => "GO",
            Language::Yaml => "YAML",
            Language::Markdown => "Markdown",
            Language::VueJs => "VueJs",
            Language::Scss => "SCSS",
            Language::Csv => "CSV",
            Language::Dockerfile => "Dockerfile",
        };
        write!(f, "{this}")
    }
}
