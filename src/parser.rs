use crate::ast::{BinOp, Expr, PiecewiseBranch, Program, Stmt, UnaryOp};
use crate::diagnostics::{Diagnostic, Span};
use crate::lexer::{Token, TokenKind};
use crate::names;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// Nesting depth of integrands, where `d x` ends the body as a differential.
    integral_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            integral_depth: 0,
        }
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

        let save = self.pos;
        if let Some((name, name_span)) = self.parse_name()? {
            let mut params = Vec::new();
            let mut is_definition_candidate = true;

            if self.at(&TokenKind::LParen) {
                self.take();
                if !self.at(&TokenKind::RParen) {
                    loop {
                        match self.parse_name()? {
                            Some((param, _)) => params.push(param),
                            None => {
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
                while self.at(&TokenKind::Newline) {
                    self.take();
                }
                let body = self.parse_expr(0)?;
                self.expect_eof_or("definition")?;
                return Ok(Stmt::Definition {
                    name,
                    params,
                    span: Span::new(name_span.start, body.span().end),
                    body,
                });
            }
        }
        self.pos = save;

        let expr = self.parse_expr(0)?;
        if self.at(&TokenKind::Assign) {
            return Err(self.error_at(
                expr.span(),
                "the left side of := must be a name or a function head; adjacent letters \
                 are a product, so multi-letter names are written \\operatorname{name}",
            ));
        }
        self.expect_eof_or("expression")?;
        Ok(Stmt::Expression(expr))
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, Diagnostic> {
        let mut lhs = self.parse_prefix()?;
        loop {
            // `x \mapsto E` binds loosest; its body extends as far as possible.
            if min_bp <= 1 && self.command_is("mapsto") {
                let Expr::Symbol { name, span } = &lhs else {
                    return Err(
                        self.error_at(lhs.span(), "\\mapsto requires a variable on its left")
                    );
                };
                let (param, start) = (name.clone(), span.start);
                self.take();
                let body = self.parse_expr(0)?;
                lhs = Expr::Lambda {
                    params: vec![param],
                    span: Span::new(start, body.span().end),
                    body: Box::new(body),
                };
                continue;
            }
            let (op, lbp, rbp) = match self.peek_kind() {
                Some(TokenKind::Eq) => (BinOp::Eq, 5, 6),
                Some(TokenKind::Lt) => (BinOp::Lt, 5, 6),
                Some(TokenKind::Command(c)) if c == "lt" => (BinOp::Lt, 5, 6),
                Some(TokenKind::Command(c)) if c == "le" || c == "leq" => (BinOp::Le, 5, 6),
                Some(TokenKind::Command(c)) if c == "gt" => (BinOp::Gt, 5, 6),
                Some(TokenKind::Command(c)) if c == "ge" || c == "geq" => (BinOp::Ge, 5, 6),
                Some(TokenKind::Command(c)) if c == "ne" || c == "neq" => (BinOp::Ne, 5, 6),
                Some(TokenKind::Command(c)) if c == "lor" || c == "vee" => (BinOp::Or, 2, 3),
                Some(TokenKind::Command(c)) if c == "land" || c == "wedge" => (BinOp::And, 3, 4),
                Some(TokenKind::Command(c)) if c == "times" => (BinOp::Mul, 20, 21),
                Some(TokenKind::Command(c)) if c == "div" => (BinOp::Div, 20, 21),
                Some(TokenKind::Le) => (BinOp::Le, 5, 6),
                Some(TokenKind::Gt) => (BinOp::Gt, 5, 6),
                Some(TokenKind::Ge) => (BinOp::Ge, 5, 6),
                Some(TokenKind::Command(c)) | Some(TokenKind::Ident(c)) if c == "in" => {
                    (BinOp::In, 5, 6)
                }
                Some(TokenKind::Command(c)) | Some(TokenKind::Ident(c)) if c == "subset" => {
                    (BinOp::Subset, 5, 6)
                }
                Some(TokenKind::Command(c)) | Some(TokenKind::Ident(c)) if c == "subseteq" => {
                    (BinOp::SubsetEq, 5, 6)
                }
                Some(TokenKind::Command(c)) | Some(TokenKind::Ident(c)) if c == "cup" => {
                    (BinOp::Union, 7, 8)
                }
                Some(TokenKind::Command(c)) | Some(TokenKind::Ident(c)) if c == "cap" => {
                    (BinOp::Intersect, 8, 9)
                }
                Some(TokenKind::Command(c)) | Some(TokenKind::Ident(c)) if c == "setminus" => {
                    (BinOp::Difference, 8, 9)
                }
                Some(TokenKind::Plus) => (BinOp::Add, 10, 11),
                Some(TokenKind::Minus) => (BinOp::Sub, 10, 11),
                Some(TokenKind::Star) => (BinOp::Mul, 20, 21),
                Some(TokenKind::Slash) => (BinOp::Div, 20, 21),
                Some(TokenKind::Command(c)) | Some(TokenKind::Ident(c)) if c == "cdot" => {
                    (BinOp::Mul, 20, 21)
                }
                Some(TokenKind::Caret) => (BinOp::Pow, 30, 30),
                Some(TokenKind::Int(_) | TokenKind::Decimal { .. })
                    if matches!(lhs, Expr::Symbol { .. }) =>
                {
                    return Err(self.error_here(
                        "a number cannot follow a name; write x_{2} for a subscripted name \
                         or 2x for a product",
                    ));
                }
                _ if self.starts_implicit_operand() && is_function_word(&lhs) => {
                    return Err(self.error_here(
                        "a function name cannot be applied by juxtaposition yet; \
                         write \\sin{x} or \\sin(x)",
                    ));
                }
                _ if self.starts_implicit_operand() => (BinOp::Mul, 20, 21),
                _ => break,
            };
            if lbp < min_bp {
                break;
            }
            let implicit = op == BinOp::Mul && self.starts_implicit_operand();
            let op_end = if implicit {
                lhs.span().end
            } else {
                self.take().span.end
            };
            let rhs = self.parse_expr(rbp)?;
            let span = Span::new(lhs.span().start, rhs.span().end.max(op_end));
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
        if let Some((name, span)) = self.parse_name()? {
            return self.parse_postfix(Expr::Symbol { name, span });
        }
        let token = self.take();
        let token_kind = match token.kind {
            TokenKind::Ident(name) if self.bare_prefix_command(&name) => TokenKind::Command(name),
            kind => kind,
        };
        let expr = match token_kind {
            TokenKind::Int(n) => Expr::Integer(n, token.span),
            // A decimal is an exact rational: 3.7 is 37/10.
            TokenKind::Decimal { digits, scale } => Expr::Rational {
                numerator: Box::new(Expr::Integer(digits, token.span)),
                denominator: Box::new(Expr::Integer(10i128.pow(scale), token.span)),
                span: token.span,
            },
            TokenKind::Ident(name) if self.bare_prefix_command(&name) => {
                self.parse_command_application(name, token.span.start)?
            }
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
            TokenKind::LBrace => self.parse_set_body(token.span.start, TokenKind::RBrace)?,
            TokenKind::SetOpen => self.parse_set_body(token.span.start, TokenKind::SetClose)?,
            TokenKind::Command(c) if c == "neg" || c == "lnot" => {
                let inner = self.parse_expr(5)?;
                Expr::Unary {
                    op: UnaryOp::Not,
                    span: Span::new(token.span.start, inner.span().end),
                    expr: Box::new(inner),
                }
            }
            TokenKind::Command(c) if c == "emptyset" || c == "varnothing" => Expr::Set {
                elements: Vec::new(),
                span: token.span,
            },
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
        self.parse_postfix(expr)
    }

    /// Function application `f(x)` applies only to names and function values
    /// (`(x \mapsto x^2)(3)`); after any other expression a parenthesis starts
    /// an implicit product, as in `2(x+1)`.
    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, Diagnostic> {
        loop {
            if self.at(&TokenKind::LParen)
                && matches!(expr, Expr::Symbol { .. } | Expr::Lambda { .. })
            {
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

    fn parse_set_body(&mut self, start: usize, close: TokenKind) -> Result<Expr, Diagnostic> {
        if let Some(TokenKind::Ident(var)) | Some(TokenKind::Command(var)) =
            self.peek_kind().cloned()
        {
            if self.tokens.get(self.pos + 1).is_some_and(|token| {
                matches!(&token.kind, TokenKind::Ident(name) | TokenKind::Command(name) if name == "in")
            }) {
                self.take();
                self.take();
                let domain = self.parse_expr(6)?;
                if self.at(&TokenKind::Pipe) || self.command_is("mid") {
                    self.take();
                    let condition = self.parse_expr(0)?;
                    let end = self.expect(close)?.end;
                    return Ok(Expr::SetComprehension {
                        var,
                        domain: Box::new(domain),
                        condition: Box::new(condition),
                        span: Span::new(start, end),
                    });
                }
                return Err(self.error_here("set comprehension requires a condition"));
            }
        }

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
        let end = self.expect(close)?.end;
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

    fn parse_matrix_body(&mut self, start: usize, environment: String) -> Result<Expr, Diagnostic> {
        let mut rows: Vec<Vec<Expr>> = Vec::new();
        let mut current: Vec<Expr> = Vec::new();

        loop {
            while self.at(&TokenKind::Newline) {
                self.take();
            }
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
            while self.at(&TokenKind::Newline) {
                self.take();
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
        let mut declared = None;

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
                    var = name.clone();
                    declared = Some(name);
                }
                lower = Some(rhs);
            } else {
                lower = Some(Box::new(group));
            }
        }
        if self.at(&TokenKind::Caret) {
            self.take();
            upper = Some(Box::new(self.parse_group_expr()?));
        }
        self.integral_depth += 1;
        let body = self.parse_expr(0);
        let differential = body.is_ok() && self.at_differential();
        self.integral_depth -= 1;
        let body = body?;
        if differential {
            self.take();
            let (differential, span) = self.parse_name()?.expect("differential variable");
            if declared.as_ref().is_some_and(|name| *name != differential) {
                return Err(self.error_at(
                    span,
                    format!(
                        "integration variable {} does not match d{}",
                        var, differential
                    ),
                ));
            }
            var = differential;
        } else if var == "x" {
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
        let var = self.parse_bound_variable("limit")?;
        match self.take().kind {
            TokenKind::Command(name) | TokenKind::Ident(name) if name == "to" => {}
            other => {
                return Err(self.error_here(format!("expected \\to in limit, got {:?}", other)))
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
            while self.at(&TokenKind::Newline) {
                self.take();
            }
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
            while self.at(&TokenKind::Newline) {
                self.take();
            }
            if self.at(&TokenKind::RowSep) {
                self.take();
            } else if !self.command_is("end") {
                return Err(self.error_here("expected row separator or \\end{cases}"));
            }
        }
    }

    fn first_non_constant_symbol(expr: &Expr) -> Option<String> {
        match expr {
            Expr::Symbol { name, .. } => Some(name.clone()),
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
            Expr::SetComprehension {
                domain, condition, ..
            } => Self::first_non_constant_symbol(domain)
                .or_else(|| Self::first_non_constant_symbol(condition)),
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
            Expr::Lambda { body, params, .. } => {
                Self::first_non_constant_symbol(body).filter(|name| !params.contains(name))
            }
            Expr::Integer(..) | Expr::Opaque { .. } => None,
        }
    }

    fn parse_product(&mut self, start: usize) -> Result<Expr, Diagnostic> {
        self.expect(TokenKind::Underscore)?;
        self.expect(TokenKind::LBrace)?;
        let var = self.parse_bound_variable("product")?;
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
        let var = self.parse_bound_variable("summation")?;
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

    fn parse_group_name(&mut self) -> Result<String, Diagnostic> {
        self.expect(TokenKind::LBrace)?;
        let mut value = String::new();
        loop {
            match self.peek_kind().cloned() {
                Some(TokenKind::Ident(part)) => value.push_str(&part),
                Some(TokenKind::Int(n)) => value.push_str(&n.to_string()),
                Some(TokenKind::Underscore) => value.push('_'),
                _ => break,
            }
            self.take();
        }
        if value.is_empty() {
            return Err(self.error_here("expected a name"));
        }
        self.expect(TokenKind::RBrace)?;
        Ok(value)
    }

    /// Parses a mathematical name if one starts here, without consuming
    /// anything otherwise: a single letter or Greek letter with an optional
    /// subscript, or an upright operator name such as `\operatorname{fact}`.
    fn parse_name(&mut self) -> Result<Option<(String, Span)>, Diagnostic> {
        let start = match self.tokens.get(self.pos) {
            Some(token) => token.span.start,
            None => return Ok(None),
        };
        let base = match self.peek_kind().cloned() {
            Some(TokenKind::Ident(letter)) if letter.len() == 1 => {
                self.take();
                letter
            }
            Some(TokenKind::Command(c)) if names::is_greek_letter(&c) => {
                self.take();
                format!("\\{}", c)
            }
            Some(TokenKind::Command(c)) | Some(TokenKind::Ident(c))
                if names::is_operator_name_command(&c)
                    && self
                        .tokens
                        .get(self.pos + 1)
                        .is_some_and(|token| token.kind == TokenKind::LBrace) =>
            {
                self.take();
                let name = self.parse_group_name()?;
                return Ok(Some((
                    format!("\\operatorname{{{}}}", name),
                    Span::new(start, self.previous_span().end),
                )));
            }
            _ => return Ok(None),
        };
        if !self.at(&TokenKind::Underscore) {
            return Ok(Some((base, Span::new(start, self.previous_span().end))));
        }
        self.take();
        let subscript = self.parse_subscript()?;
        Ok(Some((
            format!("{}_{{{}}}", base, subscript),
            Span::new(start, self.previous_span().end),
        )))
    }

    /// Subscript text of a name. Following LaTeX, an unbraced subscript is a
    /// single character; longer subscripts need braces (`x_{12}`, `x_{max}`).
    fn parse_subscript(&mut self) -> Result<String, Diagnostic> {
        let token = self.take();
        match token.kind {
            TokenKind::Int(n) if token.span.end - token.span.start == 1 => Ok(n.to_string()),
            TokenKind::Ident(letter) if letter.len() == 1 => Ok(letter),
            TokenKind::Command(c) if names::is_greek_letter(&c) => Ok(format!("\\{}", c)),
            TokenKind::Int(_) | TokenKind::Ident(_) => Err(self.error_at(
                token.span,
                "an unbraced subscript is a single character; use braces, e.g. x_{12}",
            )),
            TokenKind::LBrace => {
                let mut text = String::new();
                loop {
                    let part = self.take();
                    let piece =
                        match part.kind {
                            TokenKind::RBrace if !text.is_empty() => break,
                            TokenKind::Ident(word) => word,
                            TokenKind::Int(n) => n.to_string(),
                            TokenKind::Comma => ",".to_owned(),
                            TokenKind::Command(c) if names::is_greek_letter(&c) => {
                                format!("\\{} ", c)
                            }
                            _ => return Err(self.error_at(
                                part.span,
                                "a subscript may contain letters, digits, Greek letters and commas",
                            )),
                        };
                    text.push_str(&piece);
                }
                Ok(text.trim_end().to_owned())
            }
            _ => Err(self.error_at(token.span, "expected a subscript")),
        }
    }

    fn parse_bound_variable(&mut self, what: &str) -> Result<String, Diagnostic> {
        match self.parse_name()? {
            Some((name, _)) => Ok(name),
            None => Err(self.error_here(format!(
                "expected {} variable, got {:?}",
                what,
                self.peek_kind().cloned().unwrap_or(TokenKind::Eof)
            ))),
        }
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
        matches!(
            self.peek_kind(),
            Some(TokenKind::Command(c)) | Some(TokenKind::Ident(c)) if c == name
        )
    }

    /// `d` followed by a name inside an integrand: the differential `dx`.
    fn at_differential(&self) -> bool {
        self.integral_depth > 0
            && matches!(self.peek_kind(), Some(TokenKind::Ident(d)) if d == "d")
            && matches!(
                self.tokens.get(self.pos + 1).map(|token| &token.kind),
                Some(TokenKind::Ident(name)) if name.len() == 1
            )
    }

    /// Juxtaposed operands multiply (`2x`, `xy`, `2\pi`, `(a+b)(a-b)`).
    fn starts_implicit_operand(&self) -> bool {
        if self.at_differential() {
            return false;
        }
        match self.peek_kind() {
            Some(TokenKind::Ident(word)) | Some(TokenKind::Command(word)) => {
                !names::is_infix_word(word)
            }
            Some(TokenKind::LParen) => true,
            _ => false,
        }
    }

    fn bare_prefix_command(&self, name: &str) -> bool {
        match name {
            "frac" | "sqrt" | "abs" | "vec" | "set" | "tuple" | "dot" | "norm" | "det"
            | "transpose" | "trans" | "inverse" | "inv" | "rank" | "card" | "cardinality"
            | "trace" | "mean" | "variance" | "stdev" | "diff" | "derivative" | "subs"
            | "substitute" | "solve" | "sin" | "cos" | "tan" | "ln" | "log" | "exp" | "range"
            | "factorial" | "binom" | "choose" | "perm" | "permutation" | "gcd" | "lcm"
            | "floor" | "ceil" | "min" | "max" | "print" | "import" | "begin" | "end"
                if self.at(&TokenKind::LBrace) =>
            {
                true
            }
            "sum" | "prod" | "lim" if self.at(&TokenKind::Underscore) => true,
            "int" => true,
            _ => false,
        }
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

/// Standard function names such as `\sin` or `gcd` (but not the constant `\pi`).
fn is_function_word(expr: &Expr) -> bool {
    match expr {
        Expr::Symbol { name, .. } => {
            let word = name.strip_prefix('\\').unwrap_or(name);
            word != "pi" && names::is_reserved_word(word)
        }
        _ => false,
    }
}
