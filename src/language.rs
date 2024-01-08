#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Language {
    Rust,
    C,
    Js,
    Json,
    Python,
    Generic,
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
        };
        write!(f, "{this}")
    }
}
