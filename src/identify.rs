use std::path::Path;

use crate::language::Language;

pub fn identify(path: &Path, debug: bool) -> Language {
    if let Some(filename) = path.file_name().and_then(|x| x.to_str()) {
        match filename {
            "Dockerfile" => return Language::Dockerfile,
            "CMakeLists.txt" => return Language::CMake,
            "Makefile" => return Language::Makefile,
            "LICENSE" => return Language::Txt,
            "Cargo.lock" => return Language::Txt,
            _ => (),
        }
    }

    if let Some(extension) = path
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_lowercase())
    {
        match extension.as_str() {
            "c" | "h" | "cpp" | "hpp" => return Language::C,
            "vert" | "frag" | "glsl" => return Language::Shader,
            "rs" => return Language::Rust,
            "py" => return Language::Python,
            "js" => return Language::Js,
            "json" => return Language::Json,
            "toml" => return Language::Toml,
            "go" => return Language::Go,
            "csv" => return Language::Csv,
            "yaml" | "yml" => return Language::Yaml,
            "scss" | "css" => return Language::Css,
            "html" => return Language::Html,
            "vue" => return Language::VueJs,
            "md" => return Language::Markdown,
            "tex" | "bib" => return Language::Tex,
            "sh" | "bash" | "zsh" | "fish" => return Language::Shell,
            "txt" => return Language::Txt,
            "rb" => return Language::Ruby,
            "liquid" => return Language::Liquid,
            "slang" => return Language::Slang,
            "jpg" | "png" | "jpeg" | "gif" | "bmp" | "ttf" | "pdf" | "obj" | "mtl" | "woff"
            | "woff2" | "o" | "bin" | "gltf" | "out" | "map" | "mp3" => return Language::Asset,
            "cmake" => return Language::CMake,
            _ => (),
        }
    }

    if debug {
        println!("unidentified file type with name {}", path.display())
    }
    Language::Generic
}
