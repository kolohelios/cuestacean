//! Bounded syntax admission for the isolated native specification exporter.
use crate::model::{Document, Error, Expr, Id, Type};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Word(String),
    Text(String),
    Integer(i64),
    Symbol(u8),
    NotEqual,
    Ellipsis,
    Newline,
    End,
}

fn label(name: &str) -> Result<(), Error> {
    if name.is_empty() || name.len() > 64 {
        return Err(Error::Limit("label bytes"));
    }
    Ok(())
}

fn lex(source: &str, count: &mut usize) -> Result<Vec<Token>, Error> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        let start = pos;
        let token = match bytes[pos] {
            b' ' | b'\t' | b'\r' => {
                pos += 1;
                continue;
            }
            b'/' if bytes.get(pos + 1) == Some(&b'/') => {
                while pos < bytes.len() && bytes[pos] != b'\n' {
                    if bytes[pos] == 0 || source[pos..].starts_with('\u{feff}') {
                        return Err(Error::Unsupported("comment character"));
                    }
                    pos += source[pos..]
                        .chars()
                        .next()
                        .ok_or(Error::Syntax("comment boundary"))?
                        .len_utf8();
                }
                continue;
            }
            b'\n' => {
                pos += 1;
                Token::Newline
            }
            b'"' => {
                if bytes.get(pos..pos + 3) == Some(b"\"\"\"") {
                    return Err(Error::Unsupported("multiline string"));
                }
                pos += 1;
                loop {
                    match bytes.get(pos) {
                        Some(b'"') => {
                            pos += 1;
                            break;
                        }
                        Some(b'\\') => {
                            pos += 1;
                            match bytes.get(pos) {
                                Some(
                                    b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' | b'u',
                                ) => pos += 1,
                                Some(_) => return Err(Error::Unsupported("string escape")),
                                None => return Err(Error::Syntax("unterminated string")),
                            }
                        }
                        Some(0..=31) => return Err(Error::Syntax("string control character")),
                        Some(_) => pos += 1,
                        None => return Err(Error::Syntax("unterminated string")),
                    }
                }
                Token::Text(
                    serde_json::from_str(&source[start..pos])
                        .map_err(|_| Error::Syntax("quoted string"))?,
                )
            }
            b'-' | b'0'..=b'9' => {
                if bytes[pos] == b'-' {
                    pos += 1;
                }
                let digits = pos;
                while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
                    pos += 1;
                }
                if pos == digits {
                    return Err(Error::Unsupported("computed operation"));
                }
                if bytes
                    .get(pos)
                    .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_'))
                {
                    return Err(Error::Unsupported("numeric form"));
                }
                if pos - digits > 1 && bytes[digits] == b'0' {
                    return Err(Error::Unsupported("numeric form"));
                }
                Token::Integer(source[start..pos].parse().map_err(|_| Error::Range)?)
            }
            b'#' | b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                if bytes[pos] == b'#' {
                    pos += 1;
                }
                if !bytes
                    .get(pos)
                    .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
                {
                    return Err(Error::Syntax("definition name"));
                }
                pos += 1;
                while bytes
                    .get(pos)
                    .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
                {
                    pos += 1;
                }
                Token::Word(source[start..pos].into())
            }
            b'!' if bytes.get(pos + 1) == Some(&b'=') => {
                pos += 2;
                Token::NotEqual
            }
            b'.' if bytes.get(pos..pos + 3) == Some(b"...") => {
                pos += 3;
                Token::Ellipsis
            }
            b'&' | b'|' if bytes.get(pos + 1) == Some(&bytes[pos]) => {
                return Err(Error::Unsupported("logical operation"));
            }
            b'{' | b'}' | b'[' | b']' | b'(' | b')' | b':' | b',' | b'&' | b'|' => {
                pos += 1;
                Token::Symbol(bytes[start])
            }
            b'*' | b'?' | b'!' | b'=' | b'+' | b'/' | b'<' | b'>' | b'.' | b'\'' => {
                return Err(Error::Unsupported("syntax outside export profile"));
            }
            _ => return Err(Error::Unsupported("source character")),
        };
        *count += 1;
        if *count > 4096 {
            return Err(Error::Limit("tokens"));
        }
        tokens.push(token);
    }
    tokens.push(Token::End);
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    document: Document,
    heights: Vec<usize>,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
    fn take(&mut self) -> Token {
        let token = self.peek().clone();
        if token != Token::End {
            self.pos += 1;
        }
        token
    }
    fn symbol(&mut self, byte: u8) -> bool {
        if self.peek() == &Token::Symbol(byte) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, byte: u8) -> Result<(), Error> {
        if self.symbol(byte) {
            Ok(())
        } else {
            Err(Error::Syntax("expected punctuation"))
        }
    }
    fn newlines(&mut self) {
        while self.peek() == &Token::Newline {
            self.pos += 1;
        }
    }
    fn separators(&mut self) -> bool {
        let start = self.pos;
        self.newlines();
        self.symbol(b',');
        self.newlines();
        self.pos != start
    }
    fn node(&mut self, expr: Expr) -> Result<Id, Error> {
        if self.document.nodes.len() >= 2048 {
            return Err(Error::Limit("syntax nodes"));
        }
        let height = 1 + match &expr {
            Expr::And(a, b) | Expr::Or(a, b) => self.heights[*a].max(self.heights[*b]),
            Expr::Struct(fields) => fields
                .iter()
                .map(|(_, id)| self.heights[*id])
                .max()
                .unwrap_or(0),
            Expr::List(items, tail) => items
                .iter()
                .chain(tail.iter())
                .map(|id| self.heights[*id])
                .max()
                .unwrap_or(0),
            _ => 0,
        };
        if height > 32 {
            return Err(Error::Limit("syntax depth"));
        }
        let id = self.document.nodes.len();
        self.heights.push(height);
        self.document.nodes.push(expr);
        Ok(id)
    }
    fn expression(&mut self, depth: usize, minimum: u8) -> Result<Id, Error> {
        if depth > 32 {
            return Err(Error::Limit("syntax depth"));
        }
        let mut left = match self.take() {
            Token::Text(value) => self.node(Expr::Literal(value.into()))?,
            Token::Integer(value) => self.node(Expr::Literal(value.into()))?,
            Token::Word(word) => match word.as_str() {
                "string" => self.node(Expr::Type(Type::String))?,
                "int" => self.node(Expr::Type(Type::Int))?,
                _ if word.starts_with('#') => {
                    label(&word)?;
                    self.node(Expr::Ref(word))?
                }
                _ => return Err(Error::Unsupported("value reference or type")),
            },
            Token::NotEqual => {
                self.newlines();
                let value = match self.take() {
                    Token::Text(value) => value.into(),
                    Token::Integer(value) => value.into(),
                    Token::Word(_) => return Err(Error::Unsupported("inequality operand")),
                    _ => return Err(Error::Syntax("literal inequality operand")),
                };
                self.node(Expr::NotEqual(value))?
            }
            Token::Symbol(b'(') => {
                self.newlines();
                let id = self.expression(depth + 1, 0)?;
                self.newlines();
                self.expect(b')')?;
                id
            }
            Token::Symbol(b'{') => self.structure(depth)?,
            Token::Symbol(b'[') => self.list(depth)?,
            Token::Ellipsis => return Err(Error::Unsupported("struct ellipsis")),
            _ => return Err(Error::Syntax("expected expression")),
        };
        loop {
            let precedence = match self.peek() {
                Token::Symbol(b'&') => 2,
                Token::Symbol(b'|') => 1,
                _ => break,
            };
            if precedence < minimum {
                break;
            }
            self.take();
            self.newlines();
            let right = self.expression(depth + 1, precedence + 1)?;
            left = self.node(if precedence == 2 {
                Expr::And(left, right)
            } else {
                Expr::Or(left, right)
            })?;
        }
        if self.heights[left] > 33 - depth {
            return Err(Error::Limit("syntax depth"));
        }
        Ok(left)
    }
    fn structure(&mut self, depth: usize) -> Result<Id, Error> {
        let mut fields = Vec::new();
        self.newlines();
        while !self.symbol(b'}') {
            if fields.len() >= 64 {
                return Err(Error::Limit("container items"));
            }
            let name = match self.take() {
                Token::Text(name) => name,
                Token::Word(name) if name.starts_with('#') => {
                    return Err(Error::Unsupported("nested definition"));
                }
                Token::Word(name) if name.starts_with('_') => {
                    return Err(Error::Unsupported("hidden field"));
                }
                Token::Word(name)
                    if matches!(name.as_str(), "let" | "for" | "if" | "import" | "package") =>
                {
                    return Err(Error::Unsupported("declaration"));
                }
                Token::Word(name) => name,
                Token::Ellipsis => return Err(Error::Unsupported("struct ellipsis")),
                Token::Symbol(b'[') => return Err(Error::Unsupported("pattern field")),
                _ => return Err(Error::Syntax("field label")),
            };
            label(&name)?;
            self.expect(b':')?;
            self.newlines();
            let value = self.expression(depth + 1, 0)?;
            fields.push((name, value));
            if self.symbol(b'}') {
                break;
            }
            if !self.separators() {
                return Err(Error::Syntax("field separator"));
            }
        }
        self.node(Expr::Struct(fields))
    }
    fn list(&mut self, depth: usize) -> Result<Id, Error> {
        self.newlines();
        if self.peek() == &Token::Ellipsis {
            self.take();
            let item = self.expression(depth + 1, 0)?;
            self.newlines();
            self.expect(b']')?;
            return self.node(Expr::List(Vec::new(), Some(item)));
        }
        let mut items = Vec::new();
        while !self.symbol(b']') {
            if items.len() >= 64 {
                return Err(Error::Limit("container items"));
            }
            items.push(self.expression(depth + 1, 0)?);
            if self.symbol(b']') {
                break;
            }
            if !self.separators() {
                return Err(Error::Syntax("list separator"));
            }
            if self.peek() == &Token::Ellipsis {
                return Err(Error::Unsupported("mixed list tail syntax"));
            }
        }
        self.node(Expr::List(items, None))
    }
    fn file(&mut self) -> Result<String, Error> {
        self.newlines();
        if self.take() != Token::Word("package".into()) {
            return Err(Error::Syntax("package declaration"));
        }
        let package = match self.take() {
            Token::Word(name) if !name.starts_with('#') => name,
            _ => return Err(Error::Syntax("package name")),
        };
        if self.peek() != &Token::Newline && self.peek() != &Token::End {
            return Err(Error::Syntax("package separator"));
        }
        self.newlines();
        while self.peek() != &Token::End {
            if matches!(self.peek(), Token::Word(name) if name.starts_with('#'))
                && self.tokens.get(self.pos + 1) == Some(&Token::Symbol(b':'))
            {
                let Token::Word(name) = self.take() else {
                    unreachable!()
                };
                label(&name)?;
                self.expect(b':')?;
                self.newlines();
                if self.document.definitions.contains_key(&name) {
                    return Err(Error::Unsupported("duplicate definition"));
                }
                if self.document.definitions.len() >= 64 {
                    return Err(Error::Limit("definitions"));
                }
                let id = self.expression(1, 0)?;
                self.document.definitions.insert(name, id);
            } else {
                let id = self.expression(1, 0)?;
                self.document.roots.push(id);
            }
            if self.peek() != &Token::End && !self.separators() {
                return Err(Error::Syntax("declaration separator"));
            }
        }
        Ok(package)
    }
}

pub(crate) fn parse(schema: &str, program: &str) -> Result<Document, Error> {
    if schema
        .len()
        .checked_add(program.len())
        .is_none_or(|n| n > 65536)
    {
        return Err(Error::Limit("source bytes"));
    }
    let mut count = 0;
    let first = lex(schema, &mut count)?;
    let second = lex(program, &mut count)?;
    let mut parser = Parser {
        tokens: first,
        pos: 0,
        heights: Vec::new(),
        document: Document {
            nodes: Vec::new(),
            definitions: BTreeMap::new(),
            roots: Vec::new(),
        },
    };
    let package = parser.file()?;
    parser.tokens = second;
    parser.pos = 0;
    if parser.file()? != package {
        return Err(Error::Unsupported("different packages"));
    }
    if parser.document.roots.is_empty() {
        return Err(Error::Syntax("missing root embedding"));
    }
    for node in &parser.document.nodes {
        if let Expr::Ref(name) = node
            && !parser.document.definitions.contains_key(name)
        {
            return Err(Error::Unknown(name.clone()));
        }
    }
    Ok(parser.document)
}
