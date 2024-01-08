use std::path::Path;

use crate::language::Language;

pub fn identify(path: &Path) -> Language {
    if let Some(extension) = path
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_lowercase())
    {
        macro_rules! by_extension {
            ($lang:expr, $($id:ident),+) => {
                if [$(stringify!($id)),*].iter().any(|x| &extension == x) {
                    return $lang;
                }
            };
        }

        by_extension!(Language::C, c, h, cpp, hpp);
        by_extension!(Language::Rust, rs);
        by_extension!(Language::Python, py);
        by_extension!(Language::Js, js);
        by_extension!(Language::Json, json);
        by_extension!(Language::Toml, toml);
        by_extension!(Language::Go, go);
    }

    Language::Generic
}
