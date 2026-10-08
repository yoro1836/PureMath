use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Option<Span>,
    pub source: Option<String>,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
            source: None,
        }
    }

    pub fn at(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
            source: None,
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    fn source_context(&self, span: Span) -> Option<String> {
        let source = self.source.as_deref()?;
        let line_start = source[..span.start.min(source.len())]
            .rfind('\n')
            .map_or(0, |i| i + 1);
        let line_end = source[span.end.min(source.len())..]
            .find('\n')
            .map_or(source.len(), |i| span.end.min(source.len()) + i);
        let line = &source[line_start..line_end];
        let line_number = source[..line_start].bytes().filter(|b| *b == b'\n').count() + 1;
        let prefix_width = line_number.to_string().len();
        let caret_start = source[line_start..span.start.min(source.len())]
            .chars()
            .count();
        let caret_len = source[span.start.min(source.len())..span.end.min(source.len())]
            .chars()
            .count()
            .max(1);
        Some(format!(
            "\n  |\n{line_number:>width$} | {line}\n  | {indent:>caret_start$}{carets}",
            width = prefix_width,
            indent = "",
            caret_start = caret_start,
            carets = "^".repeat(caret_len),
        ))
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span {
            Some(span) => {
                write!(f, "{} (bytes {}..{})", self.message, span.start, span.end)?;
                if let Some(context) = self.source_context(span) {
                    write!(f, "{}", context)?;
                }
                Ok(())
            }
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for Diagnostic {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_source_context() {
        let diagnostic =
            Diagnostic::at("unexpected token", Span::new(8, 9)).with_source("x := 10\ny := + 2\n");
        let rendered = diagnostic.to_string();
        assert!(rendered.contains("2 | y := + 2"));
        assert!(rendered.contains("^"));
    }

    #[test]
    fn display_without_source_is_unchanged() {
        let diagnostic = Diagnostic::at("bad expression", Span::new(2, 5));
        assert_eq!(diagnostic.to_string(), "bad expression (bytes 2..5)");
    }
}
