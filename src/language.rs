#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    C,
    Generic,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Language::Rust => "Rust",
                Language::C => "C",
                Language::Generic => "Other",
            }
        )
    }
}
