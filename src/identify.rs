use std::{io::Read, path::Path};

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
            "zon" => return Language::Zon,
            "zig" => return Language::Zig,
            "vert" | "frag" | "glsl" => return Language::Shader,
            "rs" => return Language::Rust,
            "py" => return Language::Python,
            "js" => return Language::Javascript,
            "ts" | "tsx" => return Language::Typescript,
            "json" => return Language::Json,
            "toml" => return Language::Toml,
            "go" => return Language::Go,
            "csv" => return Language::Csv,
            "yaml" | "yml" => return Language::Yaml,
            "scss" | "css" => return Language::Css,
            "html" => return Language::Html,
            "vue" => return Language::VueJs,
            "svelte" => return Language::Svelte,
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

    // By shebang
    const SHEBANGS: &[(&str, Language)] = &[
        ("#!/bin/bash", Language::Shell),
        ("#!/bin/sh", Language::Shell),
        ("#!/usr/bin/env bash", Language::Shell),
        ("#!/usr/bin/env sh", Language::Shell),
        ("#!/usr/bin/env python", Language::Python),
        ("#!/usr/bin/env python3", Language::Python),
        ("#!/usr/bin/env node", Language::Javascript),
        ("#!/usr/bin/env deno", Language::Typescript),
    ];
    const MAX_SHEBANG_LENGTH: usize = {
        let mut max_length = 0;
        let mut i = 0;
        // Handroll this loop as const fn doesn't allow iterators or for loops??
        loop {
            if i >= SHEBANGS.len() {
                break;
            }
            let shebang = SHEBANGS[i].0;
            if shebang.len() > max_length {
                max_length = shebang.len();
            }
            i += 1;
        }
        max_length
    };
    let mut buffer = [0u8; MAX_SHEBANG_LENGTH];
    if let Ok(mut file) = std::fs::File::open(path) {
        if let Ok(n) = file.read(&mut buffer) {
            let content = std::str::from_utf8(&buffer[..n]).unwrap_or("");
            for (shebang, language) in SHEBANGS {
                if content.starts_with(shebang) {
                    return *language;
                }
            }
        }
    }

    if debug {
        println!("unidentified file type with name {}", path.display())
    }
    Language::Generic
}
