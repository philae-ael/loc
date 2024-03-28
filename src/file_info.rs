use crate::{
    identify::identify,
    language::Language,
    line_kind::{Generic, GenericWithComment, LineKind, LineKindEstimator, MultilineCommentAware},
};

#[derive(Debug, Default, Clone)]
pub struct FileInfo {
    pub textual: bool,
    pub total: usize,
    pub code: usize,
    pub comments: usize,
    pub empty: usize,
    // Number of files for this FileInfo struct
    pub file_count: usize,
}

// WARNING: default and new are not he same: default is the monoid identity, while new is now: it
// represents an empty file
impl FileInfo {
    pub fn new() -> Self {
        Self {
            textual: true,
            total: 0,
            code: 0,
            comments: 0,
            file_count: 1,
            empty: 0,
        }
    }
    pub fn new_non_text() -> Self {
        Self {
            textual: false,
            total: 0,
            code: 0,
            comments: 0,
            file_count: 1,
            empty: 0,
        }
    }

    pub fn merge_with(&mut self, other: &Self) {
        *self = Self {
            textual: self.textual || other.textual,
            total: self.total + other.total,
            code: self.code + other.code,
            empty: self.empty + other.empty,
            comments: self.comments + other.comments,
            file_count: self.file_count + other.file_count,
        }
    }
}

pub fn make_line_kind_estimator(language: Language) -> Option<Box<dyn LineKindEstimator + Send>> {
    match language {
        Language::Rust => Some(Box::new(GenericWithComment::new("//"))),
        Language::VueJs | Language::C | Language::Js | Language::Go | Language::Shader => {
            Some(Box::new(MultilineCommentAware::new("//", ["/*", "*/"])))
        }
        Language::Python => Some(Box::new(GenericWithComment::new("#"))),
        Language::Toml => Some(Box::new(GenericWithComment::new("#"))),
        Language::Markdown
        | Language::Scss
        | Language::Yaml
        | Language::Csv
        | Language::Dockerfile
        | Language::Generic
        | Language::CMake
        | Language::Json => Some(Box::new(Generic)),
        Language::Asset => None,
    }
}

pub async fn file_info_from_path(
    file: &std::path::Path,
    debug: bool,
) -> std::io::Result<(FileInfo, Language)> {
    let language = identify(file, debug);

    Ok((
        gen_file_info(file, make_line_kind_estimator(language)).await?,
        language,
    ))
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

pub async fn gen_file_info(
    file: &std::path::Path,
    line_kind_estimator: Option<Box<dyn LineKindEstimator + Send>>,
) -> std::io::Result<FileInfo> {
    let mut file_info = FileInfo::new();
    match line_kind_estimator {
        Some(mut line_kind_estimator) => {
            let f = tokio::fs::OpenOptions::new()
                .read(true)
                .write(false)
                .open(file)
                .await?;

            let mut buffered = tokio::io::BufReader::new(f);
            let mut line_buf = String::new();
            while let Some(li) = read_line(&mut buffered, &mut line_buf).await {
                file_info.total += 1;
                match line_kind_estimator.estimate(li) {
                    LineKind::Comment => file_info.comments += 1,
                    LineKind::Code => file_info.code += 1,
                    LineKind::Empty => file_info.empty += 1,
                }
            }

            Ok(file_info)
        }
        None => Ok(FileInfo::new_non_text()),
    }
}
