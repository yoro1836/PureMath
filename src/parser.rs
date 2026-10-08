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
        if self.command_is("print") {
            let start = self.take().span.start;
            let expr = self.parse_group_expr()?;
            let end = expr.span().end;
            self.expect_eof_or("print statement")?;
            return Ok(Stmt::Print {
                expr,
                span: Span::new(start, end),
            });
        }

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
            let mut is_definition_candidate = true;

            if self.at(&TokenKind::LParen) {
                self.take();
                if !self.at(&TokenKind::RParen) {
                    loop {
                        match self.peek_kind().cloned() {
                            Some(TokenKind::Ident(_)) => {
                                if let TokenKind::Ident(param) = self.take().kind {
                                    params.push(param);
                                }
                            }
                            _ => {
                                is_definition_candidate = false;
                                break;
                            }
                        }

                        if self.at(&TokenKind::Comma) {
                            self.take();
                            continue;
                        }
                        break;
                    }
                }

                if is_definition_candidate && !self.at(&TokenKind::RParen) {
                    is_definition_candidate = false;
                }

                if is_definition_candidate {
                    self.expect(TokenKind::RParen)?;
                }
            }

            if is_definition_candidate && self.at(&TokenKind::Assign) {
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
                Some(TokenKind::Command(c)) if c == "in" => (BinOp::In, 5, 6),
                Some(TokenKind::Command(c)) if c == "subset" => (BinOp::Subset, 5, 6),
                Some(TokenKind::Command(c)) if c == "subseteq" => (BinOp::SubsetEq, 5, 6),
                Some(TokenKind::Command(c)) if c == "cup" => (BinOp::Union, 7, 8),
                Some(TokenKind::Command(c)) if c == "cap" => (BinOp::Intersect, 8, 9),
                Some(TokenKind::Command(c)) if c == "setminus" => (BinOp::Difference, 8, 9),
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
                let inner = self.parse_expr(30)?;
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
            TokenKind::Command(c) if c == "prod" => self.parse_product(token.span.start)?,
            TokenKind::Command(c) if c == "int" => self.parse_integral(token.span.start)?,
            TokenKind::Command(c) if c == "lim" => self.parse_limit(token.span.start)?,
            TokenKind::Command(c) if c == "abs" => {
                let x = self.parse_group_expr()?;
                Expr::Abs {
                    span: Span::new(token.span.start, x.span().end),
                    expr: Box::new(x),
                }
            }
            TokenKind::Command(c) if c == "begin" => self.parse_environment(token.span.start)?,
            TokenKind::Command(c) => self.parse_command_application(c, token.span.start)?,
            kind => {
                return Err(
                    self.error_at(token.span, format!("expected expression, got {:?}", kind))
                )
            }
        };

        loop {
            if self.at(&TokenKind::LParen) {
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
                continue;
            }
            if self.at(&TokenKind::Bang) {
                let end = self.take().span.end;
                let callee = Expr::Symbol {
                    name: r"\factorial".to_owned(),
                    span: Span::new(end.saturating_sub(1), end),
                };
                let start = expr.span().start;
                expr = Expr::Call {
                    callee: Box::new(callee),
                    args: vec![expr],
                    span: Span::new(start, end),
                };
                continue;
            }
            break;
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

    fn parse_command_application(
        &mut self,
        command: String,
        start: usize,
    ) -> Result<Expr, Diagnostic> {
        let callee = Expr::Symbol {
            name: format!("\\{}", command),
            span: Span::new(start, start + command.len() + 1),
        };

        if matches!(command.as_str(), "vec" | "set" | "tuple") && self.at(&TokenKind::LBrace) {
            let items = self.parse_group_expr_list()?;
            if command == "vec" {
                return Ok(Expr::Vector {
                    elements: items,
                    span: Span::new(start, self.previous_span().end),
                });
            }
            if command == "set" {
                return Ok(Expr::Set {
                    elements: items,
                    span: Span::new(start, self.previous_span().end),
                });
            }
            return Ok(Expr::Call {
                callee: Box::new(callee),
                args: items,
                span: Span::new(start, self.previous_span().end),
            });
        }

        let mut args = Vec::new();
        while self.at(&TokenKind::LBrace) {
            args.push(self.parse_group_expr()?);
        }
        if args.is_empty() {
            Ok(callee)
        } else {
            Ok(Expr::Call {
                callee: Box::new(callee),
                args,
                span: Span::new(start, self.previous_span().end),
            })
        }
    }

    fn parse_group_expr_list(&mut self) -> Result<Vec<Expr>, Diagnostic> {
        self.expect(TokenKind::LBrace)?;
        let mut items = Vec::new();
        if !self.at(&TokenKind::RBrace) {
            loop {
                items.push(self.parse_expr(0)?);
                if self.at(&TokenKind::Comma) {
                    self.take();
                    continue;
                }
                break;
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(items)
    }

    fn parse_environment(&mut self, start: usize) -> Result<Expr, Diagnostic> {
        let name = self.parse_group_name()?;
        if name == "cases" {
            return self.parse_cases_body(start);
        }
        if matches!(
            name.as_str(),
            "matrix" | "pmatrix" | "bmatrix" | "Bmatrix" | "vmatrix" | "Vmatrix" | "smallmatrix"
        ) {
            return self.parse_matrix_body(start, name);
        }
        Err(self.error_here(format!("unsupported environment: {}", name)))
    }

    fn parse_matrix_body(
        &mut self,
        start: usize,
        environment: String,
    ) -> Result<Expr, Diagnostic> {
        let mut rows: Vec<Vec<Expr>> = Vec::new();
        let mut current: Vec<Expr> = Vec::new();

        loop {
            if self.command_is("end") {
                if !current.is_empty() {
                    rows.push(std::mem::take(&mut current));
                }
                self.take();
                let end_name = self.parse_group_name()?;
                if end_name != environment {
                    return Err(self.error_here("mismatched matrix environment"));
                }
                if rows.is_empty() {
                    return Err(self.error_here("matrix must contain at least one row"));
                }
                let width = rows[0].len();
                if width == 0 || rows.iter().any(|row| row.len() != width) {
                    return Err(self.error_here("matrix rows must have equal non-zero length"));
                }
                return Ok(Expr::Matrix {
                    rows,
                    span: Span::new(start, self.previous_span().end),
                });
            }

            current.push(self.parse_expr(0)?);
            if self.at(&TokenKind::Ampersand) {
                self.take();
                continue;
            }
            if self.at(&TokenKind::RowSep) {
                self.take();
                rows.push(std::mem::take(&mut current));
                continue;
            }
            if !self.command_is("end") {
                return Err(self.error_here("expected &, row separator, or \\end{matrix}"));
            }
        }
    }

    fn parse_integral(&mut self, start: usize) -> Result<Expr, Diagnostic> {
        let mut lower = None;
        let mut upper = None;
        let mut var = "x".to_owned();

        if self.at(&TokenKind::Underscore) {
            self.take();
            let group = self.parse_group_expr()?;
            if let Expr::Binary {
                op: BinOp::Eq,
                lhs,
                rhs,
                ..
            } = group
            {
                if let Expr::Symbol { name, .. } = *lhs {
                    var = name;
                }
                lower = Some(rhs);
            } else {
                lower = Some(group);
            }
        }
        if self.at(&TokenKind::Caret) {
            self.take();
            upper = Some(self.parse_group_expr()?);
        }
        let body = self.parse_expr(0)?;
        if var == "x" {
            if let Some(candidate) = Self::first_non_constant_symbol(&body) {
                var = candidate;
            }
        }
        Ok(Expr::Integral {
            var,
            lower,
            upper,
            body: Box::new(body.clone()),
            span: Span::new(start, body.span().end),
        })
    }

    fn parse_limit(&mut self, start: usize) -> Result<Expr, Diagnostic> {
        self.expect(TokenKind::Underscore)?;
        self.expect(TokenKind::LBrace)?;
        let var = match self.take().kind {
            TokenKind::Ident(name) => name,
            other => {
                return Err(
                    self.error_here(format!("expected limit variable, got {:?}", other))
                )
            }
        };
        match self.take().kind {
            TokenKind::Command(name) if name == "to" => {}
            other => {
                return Err(self.error_here(format!(
                    "expected \\to in limit, got {:?}",
                    other
                )))
            }
        }
        let target = self.parse_expr(0)?;
        self.expect(TokenKind::RBrace)?;
        let body = self.parse_expr(0)?;
        Ok(Expr::Limit {
            var,
            target: Box::new(target),
            body: Box::new(body.clone()),
            span: Span::new(start, body.span().end),
        })
    }

    fn parse_cases_body(&mut self, start: usize) -> Result<Expr, Diagnostic> {
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

    fn first_non_constant_symbol(expr: &Expr) -> Option<String> {
        match expr {
            Expr::Symbol { name, .. }
                if !matches!(name.as_str(), "\\pi" | "\\e" | "\\infty") =>
            {
                Some(name.clone())
            }
            Expr::Unary { expr, .. } => Self::first_non_constant_symbol(expr),
            Expr::Binary { lhs, rhs, .. } => Self::first_non_constant_symbol(lhs)
                .or_else(|| Self::first_non_constant_symbol(rhs)),
            Expr::Call { args, .. } => args.iter().find_map(Self::first_non_constant_symbol),
            Expr::Rational {
                numerator,
                denominator,
                ..
            } => Self::first_non_constant_symbol(numerator)
                .or_else(|| Self::first_non_constant_symbol(denominator)),
            Expr::Set { elements, .. } | Expr::Vector { elements, .. } => {
                elements.iter().find_map(Self::first_non_constant_symbol)
            }
            Expr::Matrix { rows, .. } => rows
                .iter()
                .flat_map(|row| row.iter())
                .find_map(Self::first_non_constant_symbol),
            Expr::Integral { body, .. }
            | Expr::Limit { body, .. }
            | Expr::Abs { expr: body, .. }
            | Expr::Sqrt { expr: body, .. } => Self::first_non_constant_symbol(body),
            Expr::Product { body, .. } | Expr::Sum { body, .. } => {
                Self::first_non_constant_symbol(body)
            }
            Expr::Piecewise { branches, .. } => branches.iter().find_map(|b| {
                Self::first_non_constant_symbol(&b.value)
                    .or_else(|| Self::first_non_constant_symbol(&b.condition))
            }),
            Expr::Integer(..) | Expr::Opaque { .. } => None,
        }
    }

    fn parse_product(&mut self, start: usize) -> Result<Expr, Diagnostic> {
        self.expect(TokenKind::Underscore)?;
        self.expect(TokenKind::LBrace)?;
        let var = match self.take().kind {
            TokenKind::Ident(name) => name,
            other => {
                return Err(
                    self.error_here(format!("expected product variable, got {:?}", other))
                )
            }
        };
        self.expect(TokenKind::Eq)?;
        let lower = self.parse_expr(0)?;
        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::Caret)?;
        let upper = self.parse_group_expr()?;
        let body = self.parse_expr(29)?;
        let end = body.span().end;
        Ok(Expr::Product {
            var,
            lower: Box::new(lower),
            upper: Box::new(upper),
            body: Box::new(body),
            span: Span::new(start, end),
        })
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
