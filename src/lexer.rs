use crate::diagnostics::{Diagnostic, Span};

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Int(i128),
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
    Assign,
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
    Bang,
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
                let raw = &input[number_start..i];
                let n = raw.parse::<i128>().map_err(|_| {
                    Diagnostic::at("integer literal overflow", Span::new(number_start, i))
                })?;
                out.push(Token {
                    kind: TokenKind::Int(n),
                    span: Span::new(number_start, i),
                });
            }
            c if c.is_ascii_alphabetic() => {
                let ident_start = i;
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    if c.is_ascii_alphanumeric() || c == '_' {
                        i += 1;
                    } else {
                        break;
                    }
                }

                let word = &input[ident_start..i];
                if let Some(command) = word.strip_suffix('_') {
                    if matches!(command, "sum" | "prod" | "lim" | "int") {
                        out.push(Token {
                            kind: TokenKind::Ident(command.to_owned()),
                            span: Span::new(ident_start, i - 1),
                        });
                        out.push(Token {
                            kind: TokenKind::Underscore,
                            span: Span::new(i - 1, i),
                        });
                        continue;
                    }
                }

                out.push(Token {
                    kind: TokenKind::Ident(word.to_owned()),
                    span: Span::new(ident_start, i),
                });
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
