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
        by_extension!(Language::Shader, vert, frag, glsl);
        by_extension!(Language::Rust, rs);
        by_extension!(Language::Python, py);
        by_extension!(Language::Js, js);
        by_extension!(Language::Json, json);
        by_extension!(Language::Toml, toml);
        by_extension!(Language::Go, go);
        by_extension!(Language::Csv, csv);
        by_extension!(Language::Yaml, yaml);
        by_extension!(Language::Scss, scss);
        by_extension!(Language::VueJs, vue);
        by_extension!(Language::Markdown, md);
    }

    if let Some(filename) = path.file_name().and_then(|x| x.to_str()) {
        macro_rules! by_filename {
            ($lang:expr, $($id:expr),+) => {
                if [$($id),*].iter().any(|x| &filename == x) {
                    return $lang;
                }
            };
        }

        by_filename!(Language::Dockerfile, "Dockerfile");
        by_filename!(Language::CMake, "CMakeLists.txt");
    }

    Language::Generic
}
