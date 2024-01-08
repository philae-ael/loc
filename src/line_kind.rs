pub enum LineKind {
    Comment,
    Code,
    Empty,
}

pub struct AvailableLineKinds {
    pub code: bool,
    pub comment: bool,
    pub empty: bool,
}

pub trait LineKindEstimator {
    fn estimate(&mut self, line: &str) -> LineKind;
    fn available_line_kinds(&self) -> AvailableLineKinds;
}

pub struct MultilineCommentAware {
    comment: &'static str,
    multi_line_comment: [&'static str; 2],
    is_in_multiline_comment: bool,
}

impl MultilineCommentAware {
    pub fn new(comment: &'static str, multi_line_comment: [&'static str; 2]) -> Self {
        Self {
            comment,
            multi_line_comment,
            is_in_multiline_comment: false,
        }
    }
}

impl LineKindEstimator for MultilineCommentAware {
    fn estimate(&mut self, line: &str) -> LineKind {
        let trimmed = line.trim();
        match self.is_in_multiline_comment {
            true => {
                if trimmed.ends_with(self.multi_line_comment[1]) {
                    self.is_in_multiline_comment = false;
                }
                LineKind::Comment
            }
            false => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    LineKind::Empty
                } else if trimmed.starts_with(self.comment) {
                    LineKind::Comment
                } else if trimmed.starts_with(self.multi_line_comment[0]) {
                    self.is_in_multiline_comment = true;
                    LineKind::Comment
                } else {
                    LineKind::Code
                }
            }
        }
    }

    fn available_line_kinds(&self) -> AvailableLineKinds {
        AvailableLineKinds {
            code: true,
            comment: true,
            empty: true,
        }
    }
}

pub struct GenericWithComment {
    comment: &'static str,
}

impl GenericWithComment {
    pub fn new(comment: &'static str) -> Self {
        Self { comment }
    }
}

impl LineKindEstimator for GenericWithComment {
    fn estimate(&mut self, line: &str) -> LineKind {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            LineKind::Empty
        } else if trimmed.starts_with(self.comment) {
            LineKind::Comment
        } else {
            LineKind::Code
        }
    }

    fn available_line_kinds(&self) -> AvailableLineKinds {
        AvailableLineKinds {
            code: true,
            comment: true,
            empty: true,
        }
    }
}

pub struct Generic;
impl LineKindEstimator for Generic {
    fn estimate(&mut self, line: &str) -> LineKind {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            LineKind::Empty
        } else {
            LineKind::Code
        }
    }

    fn available_line_kinds(&self) -> AvailableLineKinds {
        AvailableLineKinds {
            code: true,
            comment: false,
            empty: true,
        }
    }
}
