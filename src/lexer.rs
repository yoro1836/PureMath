use crate::diagnostics::{Diagnostic, Span};

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Int(i128),
    /// A decimal literal `digits / 10^scale`, kept exact.
    Decimal {
        digits: i128,
        scale: u32,
    },
    Ident(String),
    Command(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Underscore,
    Ampersand,
    RowSep,
    /// `\{`, the LaTeX spelling of a set's opening brace.
    SetOpen,
    /// `\}`.
    SetClose,
    Assign,
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
    Bang,
    Pipe,
    Newline,
    Eof,
}

pub fn lex(input: &str) -> Result<Vec<Token>, Diagnostic> {
    let bytes = input.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;

    while i < bytes.len() {
        let start = i;
        match bytes[i] as char {
            '\n' => {
                if paren_depth == 0 && brace_depth == 0 {
                    out.push(Token {
                        kind: TokenKind::Newline,
                        span: Span::new(i, i + 1),
                    });
                }
                i += 1;
            }
            c if c.is_ascii_whitespace() => i += 1,
            '%' => {
                while i < bytes.len() && bytes[i] as char != '\n' {
                    i += 1;
                }
            }
            '\\' => {
                if i + 1 < bytes.len() && bytes[i + 1] as char == '\\' {
                    out.push(Token {
                        kind: TokenKind::RowSep,
                        span: Span::new(i, i + 2),
                    });
                    i += 2;
                } else if i + 1 < bytes.len() && matches!(bytes[i + 1], b'{' | b'}') {
                    let opening = bytes[i + 1] == b'{';
                    if opening {
                        brace_depth += 1;
                    } else {
                        brace_depth = brace_depth.saturating_sub(1);
                    }
                    out.push(Token {
                        kind: if opening {
                            TokenKind::SetOpen
                        } else {
                            TokenKind::SetClose
                        },
                        span: Span::new(i, i + 2),
                    });
                    i += 2;
                } else if i + 1 < bytes.len()
                    && matches!(bytes[i + 1], b',' | b';' | b':' | b'!' | b' ')
                {
                    // Spacing commands carry no mathematical meaning.
                    i += 2;
                } else {
                    i += 1;
                    let command_start = i;
                    while i < bytes.len() && (bytes[i] as char).is_ascii_alphabetic() {
                        i += 1;
                    }
                    if command_start == i {
                        return Err(Diagnostic::at(
                            "expected LaTeX command after backslash",
                            Span::new(start, i),
                        ));
                    }
                    let command = input[command_start..i].to_owned();
                    if is_presentation_command(&command) {
                        // `\left.` / `\right.` use `.` as an empty delimiter.
                        if matches!(command.as_str(), "left" | "right")
                            && i < bytes.len()
                            && bytes[i] == b'.'
                        {
                            i += 1;
                        }
                        continue;
                    }
                    out.push(Token {
                        kind: TokenKind::Command(command),
                        span: Span::new(start, i),
                    });
                }
            }
            ':' if i + 1 < bytes.len() && bytes[i + 1] as char == '=' => {
                out.push(Token {
                    kind: TokenKind::Assign,
                    span: Span::new(i, i + 2),
                });
                i += 2;
            }
            '=' => {
                out.push(Token {
                    kind: TokenKind::Eq,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '<' if i + 1 < bytes.len() && bytes[i + 1] as char == '=' => {
                out.push(Token {
                    kind: TokenKind::Le,
                    span: Span::new(i, i + 2),
                });
                i += 2;
            }
            '>' if i + 1 < bytes.len() && bytes[i + 1] as char == '=' => {
                out.push(Token {
                    kind: TokenKind::Ge,
                    span: Span::new(i, i + 2),
                });
                i += 2;
            }
            '<' => {
                out.push(Token {
                    kind: TokenKind::Lt,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '>' => {
                out.push(Token {
                    kind: TokenKind::Gt,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '!' => {
                out.push(Token {
                    kind: TokenKind::Bang,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '|' => {
                out.push(Token {
                    kind: TokenKind::Pipe,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '+' => {
                out.push(Token {
                    kind: TokenKind::Plus,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '-' => {
                out.push(Token {
                    kind: TokenKind::Minus,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '*' => {
                out.push(Token {
                    kind: TokenKind::Star,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '/' => {
                out.push(Token {
                    kind: TokenKind::Slash,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '^' => {
                out.push(Token {
                    kind: TokenKind::Caret,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '(' => {
                out.push(Token {
                    kind: TokenKind::LParen,
                    span: Span::new(i, i + 1),
                });
                paren_depth += 1;
                i += 1;
            }
            ')' => {
                out.push(Token {
                    kind: TokenKind::RParen,
                    span: Span::new(i, i + 1),
                });
                paren_depth = paren_depth.saturating_sub(1);
                i += 1;
            }
            '{' => {
                out.push(Token {
                    kind: TokenKind::LBrace,
                    span: Span::new(i, i + 1),
                });
                brace_depth += 1;
                i += 1;
            }
            '}' => {
                out.push(Token {
                    kind: TokenKind::RBrace,
                    span: Span::new(i, i + 1),
                });
                brace_depth = brace_depth.saturating_sub(1);
                i += 1;
            }
            ',' => {
                out.push(Token {
                    kind: TokenKind::Comma,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '_' => {
                out.push(Token {
                    kind: TokenKind::Underscore,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            '&' => {
                out.push(Token {
                    kind: TokenKind::Ampersand,
                    span: Span::new(i, i + 1),
                });
                i += 1;
            }
            c if c.is_ascii_digit() => {
                let number_start = i;
                while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                    i += 1;
                }
                let mut scale = 0u32;
                if i + 1 < bytes.len() && bytes[i] == b'.' && bytes[i + 1].is_ascii_digit() {
                    i += 1;
                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                        scale += 1;
                    }
                }
                let digits: String = input[number_start..i]
                    .chars()
                    .filter(char::is_ascii_digit)
                    .collect();
                let n = digits.parse::<i128>().map_err(|_| {
                    Diagnostic::at("numeric literal overflow", Span::new(number_start, i))
                })?;
                if scale > 0 && 10i128.checked_pow(scale).is_none() {
                    return Err(Diagnostic::at(
                        "numeric literal overflow",
                        Span::new(number_start, i),
                    ));
                }
                out.push(Token {
                    kind: if scale == 0 {
                        TokenKind::Int(n)
                    } else {
                        TokenKind::Decimal { digits: n, scale }
                    },
                    span: Span::new(number_start, i),
                });
            }
            c if c.is_ascii_alphabetic() => {
                let word_start = i;
                while i < bytes.len() && (bytes[i] as char).is_ascii_alphabetic() {
                    i += 1;
                }
                let word = &input[word_start..i];
                if word.len() == 1 || crate::names::is_reserved_word(word) {
                    out.push(Token {
                        kind: TokenKind::Ident(word.to_owned()),
                        span: Span::new(word_start, i),
                    });
                } else {
                    // Adjacent letters are separate single-letter names (`xy` is x·y).
                    for (offset, letter) in word.char_indices() {
                        out.push(Token {
                            kind: TokenKind::Ident(letter.to_string()),
                            span: Span::new(word_start + offset, word_start + offset + 1),
                        });
                    }
                }
            }
            _ => {
                return Err(Diagnostic::at(
                    format!("unexpected character {:?}", bytes[i] as char),
                    Span::new(i, i + 1),
                ))
            }
        }
    }

    out.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(input.len(), input.len()),
    });
    Ok(out)
}

/// Delimiter sizing and spacing commands affect only typesetting.
fn is_presentation_command(command: &str) -> bool {
    matches!(
        command,
        "left"
            | "right"
            | "big"
            | "Big"
            | "bigg"
            | "Bigg"
            | "bigl"
            | "bigr"
            | "Bigl"
            | "Bigr"
            | "quad"
            | "qquad"
    )
}
