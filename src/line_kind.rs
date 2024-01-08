pub enum LineKind {
    Comment,
    Code,
    Empty,
}

pub const fn generic_line_kind_with_comment(comment: &'static str) -> impl (Fn(&str) -> LineKind) {
    move |line: &str| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            LineKind::Empty
        } else if trimmed.starts_with(comment) {
            LineKind::Comment
        } else {
            LineKind::Code
        }
    }
}

pub const fn generic_line_kind() -> impl (Fn(&str) -> LineKind) {
    move |line: &str| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            LineKind::Empty
        } else {
            LineKind::Code
        }
    }
}
