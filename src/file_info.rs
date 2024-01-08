use std::path::PathBuf;

use crate::{
    identify::identify,
    language::Language,
    line_kind::{generic_line_kind, generic_line_kind_with_comment, LineKind},
};

#[derive(Debug, Default, Clone)]
pub struct FileInfo {
    pub total: usize,
    pub code: usize,
    pub comments: usize,
    pub empty: usize,
    // Number of files for this FileInfo struct
    pub file_count: usize,
}

// WARNING: default and are not he same: default is the monoid identity, while new is now: it
// represents an empty file
impl FileInfo {
    pub fn new() -> Self {
        Self {
            total: 0,
            code: 0,
            comments: 0,
            file_count: 1,
            empty: 0,
        }
    }

    pub fn merge_with(&mut self, other: &Self) {
        *self = Self {
            total: self.total + other.total,
            code: self.code + other.code,
            empty: self.empty + other.empty,
            comments: self.comments + other.comments,
            file_count: self.file_count + other.file_count,
        }
    }
}

pub async fn file_info_from_path(file: PathBuf) -> std::io::Result<(FileInfo, Language)> {
    let language = identify(&file);
    macro_rules! with_line_kind {
        ($line_kind:expr) => {
            gen_file_info(file, $line_kind).await
        };
    }

    #[rustfmt::skip]
    let file_info = match language {
        Language::Rust    => with_line_kind!(generic_line_kind_with_comment("//")),
        Language::C       => with_line_kind!(generic_line_kind_with_comment("//")),
        Language::Generic => with_line_kind!(generic_line_kind()),
    }?;

    Ok((file_info, language))
}

///  ---

async fn read_line<'a, T: tokio::io::AsyncRead + std::marker::Unpin>(
    buf: &mut tokio::io::BufReader<T>,
    s: &'a mut String,
) -> Option<&'a str> {
    s.clear();

    use tokio::io::AsyncBufReadExt;
    match buf.read_line(s).await {
        Err(_) => None,
        Ok(0) => None,
        Ok(_n) => Some(s),
    }
}

pub async fn gen_file_info<F: Fn(&str) -> LineKind>(
    file: PathBuf,
    line_kind: F,
) -> std::io::Result<FileInfo> {
    let mut file_info = FileInfo::new();
    let f = tokio::fs::OpenOptions::new()
        .read(true)
        .write(false)
        .open(file)
        .await?;

    let mut buffered = tokio::io::BufReader::new(f);
    let mut line_buf = String::new();
    while let Some(li) = read_line(&mut buffered, &mut line_buf).await {
        file_info.total += 1;
        match line_kind(li) {
            LineKind::Comment => file_info.comments += 1,
            LineKind::Code => file_info.code += 1,
            LineKind::Empty => file_info.empty += 1,
        }
    }

    Ok(file_info)
}
