use crate::ast::{BinOp, Expr, PiecewiseBranch, Program, Stmt, UnaryOp};
use crate::diagnostics::{Diagnostic, Span};
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        let mut statements = Vec::new();
        while !self.at(&TokenKind::Eof) {
            while self.at(&TokenKind::Newline) {
                self.take();
            }
            if self.at(&TokenKind::Eof) {
                break;
            }
            statements.push(self.parse_statement()?);
            while self.at(&TokenKind::Newline) {
                self.take();
            }
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Stmt, Diagnostic> {
        if self.command_is("import") {
            let start = self.take().span.start;
            let module = self.parse_group_name()?;
            let end = self.previous_span().end;
            self.expect_eof_or("import statement")?;
            return Ok(Stmt::Import {
                module,
                span: Span::new(start, end),
            });
        }

        if let Some(TokenKind::Ident(name)) = self.peek_kind().cloned() {
            let save = self.pos;
            let name_span = self.take().span;
            let mut params = Vec::new();
            if self.at(&TokenKind::LParen) {
                self.take();
                if !self.at(&TokenKind::RParen) {
                    loop {
                        match self.take().kind {
                            TokenKind::Ident(p) => params.push(p),
                            kind => {
                                return Err(self.error_here(format!(
                                    "expected parameter name, got {:?}",
                                    kind
                                )))
                            }
                        }
                        if self.at(&TokenKind::Comma) {
                            self.take();
                            continue;
                        }
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
            }

            if self.at(&TokenKind::Assign) {
                self.take();
                let body = self.parse_expr(0)?;
                self.expect_eof_or("definition")?;
                return Ok(Stmt::Definition {
                    name,
                    params,
                    span: Span::new(name_span.start, body.span().end),
                    body,
                });
            }
            self.pos = save;
        }

        let expr = self.parse_expr(0)?;
        self.expect_eof_or("expression")?;
        Ok(Stmt::Expression(expr))
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, Diagnostic> {
        let mut lhs = self.parse_prefix()?;
        loop {
            let (op, lbp, rbp) = match self.peek_kind() {
                Some(TokenKind::Eq) => (BinOp::Eq, 5, 6),
                Some(TokenKind::Lt) => (BinOp::Lt, 5, 6),
                Some(TokenKind::Le) => (BinOp::Le, 5, 6),
                Some(TokenKind::Gt) => (BinOp::Gt, 5, 6),
                Some(TokenKind::Ge) => (BinOp::Ge, 5, 6),
                Some(TokenKind::Plus) => (BinOp::Add, 10, 11),
                Some(TokenKind::Minus) => (BinOp::Sub, 10, 11),
                Some(TokenKind::Star) => (BinOp::Mul, 20, 21),
                Some(TokenKind::Slash) => (BinOp::Div, 20, 21),
                Some(TokenKind::Command(c)) if c == "cdot" => (BinOp::Mul, 20, 21),
                Some(TokenKind::Caret) => (BinOp::Pow, 30, 30),
                _ => break,
            };
            if lbp < min_bp {
                break;
            }
            let op_span = self.take().span;
            let rhs = self.parse_expr(rbp)?;
            let span = Span::new(lhs.span().start, rhs.span().end.max(op_span.end));
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            };
        }
        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.take();
        let mut expr = match token.kind {
            TokenKind::Int(n) => Expr::Integer(n, token.span),
            TokenKind::Ident(name) => Expr::Symbol {
                name,
                span: token.span,
            },
            TokenKind::Minus => {
                let inner = self.parse_expr(31)?;
                Expr::Unary {
                    op: UnaryOp::Neg,
                    span: Span::new(token.span.start, inner.span().end),
                    expr: Box::new(inner),
                }
            }
            TokenKind::LParen => {
                let inner = self.parse_expr(0)?;
                self.expect(TokenKind::RParen)?;
                inner
            }
            TokenKind::LBrace => self.parse_set_body(token.span.start)?,
            TokenKind::Command(c) if c == "frac" => {
                let a = self.parse_group_expr()?;
                let b = self.parse_group_expr()?;
                Expr::Rational {
                    span: Span::new(token.span.start, b.span().end),
                    numerator: Box::new(a),
                    denominator: Box::new(b),
                }
            }
            TokenKind::Command(c) if c == "sqrt" => {
                let x = self.parse_group_expr()?;
                Expr::Sqrt {
                    span: Span::new(token.span.start, x.span().end),
                    expr: Box::new(x),
                }
            }
            TokenKind::Command(c) if c == "sum" => self.parse_sum(token.span.start)?,
            TokenKind::Command(c) if c == "begin" => self.parse_cases(token.span.start)?,
            TokenKind::Command(c) => Expr::Opaque {
                text: format!("\\{}", c),
                span: token.span,
            },
            kind => {
                return Err(
                    self.error_at(token.span, format!("expected expression, got {:?}", kind))
                )
            }
        };

        loop {
            if !self.at(&TokenKind::LParen) {
                break;
            }
            let start = self.take().span.start;
            let mut args = Vec::new();
            if !self.at(&TokenKind::RParen) {
                loop {
                    args.push(self.parse_expr(0)?);
                    if self.at(&TokenKind::Comma) {
                        self.take();
                        continue;
                    }
                    break;
                }
            }
            let end = self.expect(TokenKind::RParen)?.end;
            let expr_start = expr.span().start.min(start);
            expr = Expr::Call {
                callee: Box::new(expr),
                args,
                span: Span::new(expr_start, end),
            };
        }
        Ok(expr)
    }

    fn parse_set_body(&mut self, start: usize) -> Result<Expr, Diagnostic> {
        let mut elements = Vec::new();
        if !self.at(&TokenKind::RBrace) {
            loop {
                elements.push(self.parse_expr(0)?);
                if self.at(&TokenKind::Comma) {
                    self.take();
                    continue;
                }
                break;
            }
        }
        let end = self.expect(TokenKind::RBrace)?.end;
        Ok(Expr::Set {
            elements,
            span: Span::new(start, end),
        })
    }

    fn parse_group_expr(&mut self) -> Result<Expr, Diagnostic> {
        self.expect(TokenKind::LBrace)?;
        let expr = self.parse_expr(0)?;
        self.expect(TokenKind::RBrace)?;
        Ok(expr)
    }

    fn parse_sum(&mut self, start: usize) -> Result<Expr, Diagnostic> {
        self.expect(TokenKind::Underscore)?;
        self.expect(TokenKind::LBrace)?;
        let var = match self.take().kind {
            TokenKind::Ident(name) => name,
            other => {
                return Err(self.error_here(format!("expected summation variable, got {:?}", other)))
            }
        };
        self.expect(TokenKind::Eq)?;
        let lower = self.parse_expr(0)?;
        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::Caret)?;
        let upper = self.parse_group_expr()?;
        let body = self.parse_expr(29)?;
        let end = body.span().end;
        Ok(Expr::Sum {
            var,
            lower: Box::new(lower),
            upper: Box::new(upper),
            body: Box::new(body),
            span: Span::new(start, end),
        })
    }

    fn parse_cases(&mut self, start: usize) -> Result<Expr, Diagnostic> {
        let name = self.parse_group_name()?;
        if name != "cases" {
            return Err(self.error_here(format!("unsupported environment: {}", name)));
        }
        let mut branches = Vec::new();
        loop {
            if self.command_is("end") {
                self.take();
                let end_name = self.parse_group_name()?;
                if end_name != "cases" {
                    return Err(self.error_here("mismatched cases environment"));
                }
                let end = self.previous_span().end;
                return Ok(Expr::Piecewise {
                    branches,
                    span: Span::new(start, end),
                });
            }
            let value = self.parse_expr(0)?;
            self.expect(TokenKind::Ampersand)?;
            let condition = self.parse_expr(0)?;
            let end = condition.span().end;
            branches.push(PiecewiseBranch {
                value,
                condition,
                span: Span::new(
                    branches
                        .last()
                        .map_or(start, |b: &PiecewiseBranch| b.span.end),
                    end,
                ),
            });
            if self.at(&TokenKind::RowSep) {
                self.take();
            } else if !self.command_is("end") {
                return Err(self.error_here("expected row separator or \\end{cases}"));
            }
        }
    }

    fn parse_group_name(&mut self) -> Result<String, Diagnostic> {
        self.expect(TokenKind::LBrace)?;
        let value = match self.take().kind {
            TokenKind::Ident(s) | TokenKind::Command(s) => s,
            other => return Err(self.error_here(format!("expected group name, got {:?}", other))),
        };
        self.expect(TokenKind::RBrace)?;
        Ok(value)
    }

    fn expect_eof_or(&self, what: &str) -> Result<(), Diagnostic> {
        if self.at(&TokenKind::Eof) || self.at(&TokenKind::Newline) {
            Ok(())
        } else {
            Err(self.error_here(format!("unexpected tokens after {}", what)))
        }
    }

    fn expect(&mut self, want: TokenKind) -> Result<Span, Diagnostic> {
        let token = self.take();
        if token.kind == want {
            Ok(token.span)
        } else {
            Err(self.error_at(
                token.span,
                format!("expected {:?}, got {:?}", want, token.kind),
            ))
        }
    }

    fn take(&mut self) -> Token {
        let token = self.tokens.get(self.pos).cloned().unwrap_or(Token {
            kind: TokenKind::Eof,
            span: Span::default(),
        });
        self.pos += 1;
        token
    }

    fn previous_span(&self) -> Span {
        self.tokens
            .get(self.pos.saturating_sub(1))
            .map(|t| t.span)
            .unwrap_or_default()
    }
    fn peek_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos).map(|t| &t.kind)
    }
    fn at(&self, want: &TokenKind) -> bool {
        self.peek_kind() == Some(want)
    }
    fn command_is(&self, name: &str) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Command(c)) if c == name)
    }
    fn error_here(&self, message: impl Into<String>) -> Diagnostic {
        Diagnostic::at(
            message,
            self.tokens
                .get(self.pos)
                .map(|t| t.span)
                .unwrap_or_default(),
        )
    }
    fn error_at(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::at(message, span)
    }
}
