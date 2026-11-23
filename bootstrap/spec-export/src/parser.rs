//! Bounded syntax admission for the isolated native specification exporter.
use crate::{
    bindings::{self, Binding, Scope},
    model::{Document, Error, Expr, Id, StringSlot, Template},
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Word(String),
    Text(String),
    Multiline(String),
    Interpolation {
        prefix: String,
        name: String,
        suffix: String,
    },
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

// Decode literal DATA only; decoded backslashes are never scanned for syntax.
fn string_segment(source: &str) -> Result<String, Error> {
    serde_json::from_str(&format!("\"{source}\"")).map_err(|_| Error::Syntax("quoted string"))
}

// Validate the raw body and exact closing margin before allocating decoded DATA.
// Source admission already bounds this entire scan to the combined 64 KiB input.
fn multiline(source: &str, pos: &mut usize) -> Result<String, Error> {
    let bytes = source.as_bytes();
    *pos += 3;
    if bytes.get(*pos) != Some(&b'\n') {
        return Err(Error::Unsupported("multiline opening newline"));
    }
    *pos += 1;
    let start = *pos;
    let mut line = start;
    let margin = loop {
        let mut first = line;
        while matches!(bytes.get(first), Some(b' ' | b'\t')) {
            first += 1;
        }
        loop {
            if bytes.get(*pos..*pos + 3) == Some(b"\"\"\"") {
                if *pos != first {
                    return Err(Error::Unsupported("multiline delimiter position"));
                }
                break;
            }
            match bytes.get(*pos) {
                Some(b'\n') => {
                    *pos += 1;
                    line = *pos;
                    break;
                }
                Some(b'\\') => return Err(Error::Unsupported("multiline backslash")),
                Some(0..=8 | 11..=31 | 127) => {
                    return Err(Error::Unsupported("multiline control character"));
                }
                Some(_) if bytes.get(*pos..*pos + 3) == Some(b"\xef\xbb\xbf") => {
                    return Err(Error::Unsupported("multiline byte order mark"));
                }
                Some(_) => *pos += 1,
                None => return Err(Error::Syntax("unterminated multiline string")),
            }
        }
        if bytes.get(*pos..*pos + 3) == Some(b"\"\"\"") && *pos == first {
            break &source[line..*pos];
        }
    };
    let end = if line == start { start } else { line - 1 };
    let body = &source[start..end];
    let mut length = 0usize;
    for (index, line) in body.split('\n').enumerate() {
        let data = if line.is_empty() {
            line
        } else {
            line.strip_prefix(margin)
                .ok_or(Error::Unsupported("multiline indentation"))?
        };
        length = length
            .checked_add(usize::from(index != 0))
            .and_then(|n| n.checked_add(data.len()))
            .filter(|n| *n <= source.len())
            .ok_or(Error::Limit("source bytes"))?;
    }
    let mut decoded = String::with_capacity(length);
    for (index, line) in body.split('\n').enumerate() {
        if index != 0 {
            decoded.push('\n');
        }
        if !line.is_empty() {
            // The first pass validated every prefix and the complete byte length.
            decoded.push_str(&line[margin.len()..]);
        }
    }
    *pos += 3;
    Ok(decoded)
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
            b'"' if bytes.get(pos..pos + 3) == Some(b"\"\"\"") => {
                Token::Multiline(multiline(source, &mut pos)?)
            }
            b'"' => {
                pos += 1;
                let mut segment = pos;
                let mut interpolation = None;
                loop {
                    match bytes.get(pos) {
                        Some(b'"') => {
                            pos += 1;
                            break;
                        }
                        Some(b'\\') if bytes.get(pos + 1) == Some(&b'(') => {
                            if interpolation.is_some() {
                                return Err(Error::Unsupported("multiple string interpolations"));
                            }
                            let prefix = string_segment(&source[segment..pos])?;
                            pos += 2;
                            let name = pos;
                            if !bytes
                                .get(pos)
                                .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
                            {
                                return Err(Error::Unsupported("string interpolation name"));
                            }
                            pos += 1;
                            while bytes
                                .get(pos)
                                .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
                            {
                                pos += 1;
                            }
                            if bytes.get(pos) != Some(&b')') {
                                return Err(Error::Unsupported("string interpolation name"));
                            }
                            interpolation = Some((prefix, source[name..pos].to_owned()));
                            pos += 1;
                            segment = pos;
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
                if let Some((prefix, name)) = interpolation {
                    Token::Interpolation {
                        prefix,
                        name,
                        suffix: string_segment(&source[segment..pos - 1])?,
                    }
                } else {
                    Token::Text(
                        serde_json::from_str(&source[start..pos])
                            .map_err(|_| Error::Syntax("quoted string"))?,
                    )
                }
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
                let word = &source[start..pos];
                // Adjacent _| is the beginning of bottom syntax, not Top
                // followed by a disjunction. Keep the inherited rejection.
                if word == "_" && bytes.get(pos) == Some(&b'|') {
                    return Err(Error::Unsupported("value reference or type"));
                }
                Token::Word(word.into())
            }
            b'!' if bytes.get(pos + 1) == Some(&b'=') => {
                pos += 2;
                Token::NotEqual
            }
            b'.' if bytes.get(pos..pos + 3) == Some(b"...") => {
                pos += 3;
                Token::Ellipsis
            }
            b'.' => {
                if !matches!(tokens.last(), Some(Token::Word(name)) if name.starts_with('#'))
                    || source[..pos].trim_end_matches([' ', '\t']).ends_with('\r')
                    || source[pos + 1..]
                        .trim_start_matches([' ', '\t'])
                        .starts_with('\r')
                {
                    return Err(Error::Unsupported("direct scalar projection required"));
                }
                pos += 1;
                Token::Symbol(b'.')
            }
            b'&' | b'|' if bytes.get(pos + 1) == Some(&bytes[pos]) => {
                return Err(Error::Unsupported("logical operation"));
            }
            b'{' | b'}' | b'[' | b']' | b'(' | b')' | b':' | b',' | b'&' | b'|' | b'=' | b'*'
            | b'+' => {
                pos += 1;
                Token::Symbol(bytes[start])
            }
            b'?' | b'!' | b'/' | b'<' | b'>' | b'\'' => {
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
    scopes: Vec<Scope>,
    bindings: Vec<Binding>,
    scope: usize,
    initializers: Vec<usize>,
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
    fn syntax(&mut self) -> Result<(), Error> {
        if self.document.syntax >= 2048 {
            return Err(Error::Limit("syntax nodes"));
        }
        self.document.syntax += 1;
        Ok(())
    }
    fn node(&mut self, expr: Expr) -> Result<Id, Error> {
        self.syntax()?;
        let height = 1 + match &expr {
            Expr::And(a, b) | Expr::Or(a, b) | Expr::Concat(a, b) => {
                self.heights[*a].max(self.heights[*b])
            }
            Expr::Marked(id) => self.heights[*id],
            Expr::Comprehension { template, .. } => template.height(),
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
            Token::Multiline(_) if self.peek() == &Token::Symbol(b':') => {
                return Err(Error::Unsupported("multiline field label"));
            }
            Token::Text(value) | Token::Multiline(value) => {
                self.node(Expr::Literal(value.into()))?
            }
            Token::Integer(value) => self.node(Expr::Literal(value.into()))?,
            Token::Word(word) => match word.as_str() {
                _ if word.starts_with('#') => {
                    label(&word)?;
                    if self.symbol(b'.') {
                        let Token::Word(field) = self.take() else {
                            return Err(Error::Unsupported("regular scalar projection field"));
                        };
                        if field.starts_with(['#', '_']) {
                            return Err(Error::Unsupported("regular scalar projection field"));
                        }
                        label(&field)?;
                        self.document.projections += 1;
                        self.node(Expr::Projection {
                            definition: word,
                            field,
                        })?
                    } else {
                        self.node(Expr::Ref(word))?
                    }
                }
                "true" => self.node(Expr::Literal(true.into()))?,
                "false" => self.node(Expr::Literal(false.into()))?,
                "_" if self.document.marked => self.node(Expr::Top)?,
                "let" | "for" | "if" | "in" | "import" | "package" | "null" | "_" => {
                    return Err(Error::Unsupported("value reference or type"));
                }
                _ => {
                    label(&word)?;
                    self.node(Expr::Name {
                        scope: self.scope,
                        excluded: self.initializers.clone(),
                        name: word,
                    })?
                }
            },
            Token::Interpolation { .. } => {
                return Err(Error::Unsupported("key interpolation context"));
            }
            Token::NotEqual => {
                self.newlines();
                let value = match self.take() {
                    Token::Text(value) => value.into(),
                    Token::Integer(value) => value.into(),
                    Token::Interpolation { prefix, suffix, .. }
                        if !prefix.is_empty() || !suffix.is_empty() =>
                    {
                        return Err(Error::Unsupported("string interpolation context"));
                    }
                    Token::Multiline(_) => {
                        return Err(Error::Unsupported("multiline inequality operand"));
                    }
                    Token::Word(_) => return Err(Error::Unsupported("inequality operand")),
                    _ => return Err(Error::Syntax("literal inequality operand")),
                };
                self.node(Expr::NotEqual(value))?
            }
            Token::Symbol(b'*') => {
                self.newlines();
                let operand = self.expression(depth + 1, 3)?;
                self.node(Expr::Marked(operand))?
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
                Token::Symbol(b'+') => 3,
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
            let expression = match precedence {
                3 => {
                    self.document.concatenations += 1;
                    Expr::Concat(left, right)
                }
                2 => Expr::And(left, right),
                _ => Expr::Or(left, right),
            };
            left = self.node(expression)?;
        }
        if self.heights[left] > 33 - depth {
            return Err(Error::Limit("syntax depth"));
        }
        Ok(left)
    }
    fn enter_scope(&mut self, parent: Option<usize>) {
        self.scope = self.scopes.len();
        self.scopes.push(Scope {
            parent,
            bindings: BTreeMap::new(),
            fields: Default::default(),
            excluded: self.initializers.clone(),
        });
    }
    fn binding(&mut self, depth: usize) -> Result<(), Error> {
        self.take(); // let
        let name = match self.take() {
            Token::Word(name)
                if !name.starts_with('#')
                    && !matches!(
                        name.as_str(),
                        "let"
                            | "for"
                            | "if"
                            | "in"
                            | "import"
                            | "package"
                            | "true"
                            | "false"
                            | "null"
                    ) =>
            {
                name
            }
            _ => return Err(Error::Unsupported("binding name")),
        };
        label(&name)?;
        if self.scopes[self.scope].bindings.contains_key(&name) {
            return Err(Error::Unsupported("duplicate binding"));
        }
        if self.bindings.len() >= 64 {
            return Err(Error::Limit("bindings"));
        }
        self.expect(b'=')?;
        self.newlines();
        let index = self.bindings.len();
        self.scopes[self.scope].bindings.insert(name.clone(), index);
        self.bindings.push(Binding {
            name,
            scope: self.scope,
            body: None,
        });
        self.initializers.push(index);
        let body = self.expression(depth, 0)?;
        self.initializers.pop();
        self.bindings[index].body = Some(body);
        Ok(())
    }
    fn structure(&mut self, depth: usize) -> Result<Id, Error> {
        let parent = self.scope;
        self.enter_scope(Some(parent));
        let mut fields = Vec::new();
        self.newlines();
        if self.peek() == &Token::Word("for".into()) {
            let id = self.comprehension(depth)?;
            self.scope = parent;
            return Ok(id);
        }
        while !self.symbol(b'}') {
            if self.peek() == &Token::Word("let".into()) {
                self.binding(depth + 1)?;
                if self.symbol(b'}') {
                    break;
                }
                if !self.separators() {
                    return Err(Error::Syntax("field separator"));
                }
                continue;
            }
            if fields.len() >= 64 {
                return Err(Error::Limit("container items"));
            }
            let name = match self.take() {
                Token::Text(name) => name,
                Token::Multiline(_) => return Err(Error::Unsupported("multiline field label")),
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
                Token::Word(name) => {
                    self.scopes[self.scope].fields.insert(name.clone());
                    name
                }
                Token::Interpolation { .. } => {
                    return Err(Error::Unsupported("key interpolation context"));
                }
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
        self.scope = parent;
        self.node(Expr::Struct(fields))
    }
    fn iterator(&mut self, blank: bool) -> Result<Option<String>, Error> {
        match self.take() {
            Token::Word(name) if blank && name == "_" => Ok(None),
            Token::Word(name)
                if !name.starts_with('#')
                    && !matches!(
                        name.as_str(),
                        "_" | "let"
                            | "for"
                            | "if"
                            | "in"
                            | "import"
                            | "package"
                            | "true"
                            | "false"
                            | "null"
                    ) =>
            {
                label(&name)?;
                Ok(Some(name))
            }
            _ => Err(Error::Unsupported("iterator name")),
        }
    }
    fn comprehension(&mut self, depth: usize) -> Result<Id, Error> {
        self.take(); // for
        let key = self
            .iterator(false)?
            .ok_or(Error::Unsupported("blank key"))?;
        if !self.symbol(b',') {
            return Err(Error::Unsupported("two iterators required"));
        }
        let value = self.iterator(true)?;
        if value.as_ref() == Some(&key) {
            return Err(Error::Unsupported("duplicate iterator"));
        }
        if self.take() != Token::Word("in".into()) {
            return Err(Error::Unsupported("comprehension clause"));
        }
        let source = match self.take() {
            Token::Word(name)
                if !name.starts_with('#')
                    && !matches!(
                        name.as_str(),
                        "_" | "true"
                            | "false"
                            | "null"
                            | "for"
                            | "if"
                            | "let"
                            | "in"
                            | "import"
                            | "package"
                    ) =>
            {
                label(&name)?;
                self.node(Expr::Name {
                    scope: self.scope,
                    excluded: self.initializers.clone(),
                    name,
                })?
            }
            _ => return Err(Error::Unsupported("comprehension source")),
        };
        if !self.symbol(b'{') {
            return Err(Error::Unsupported("comprehension clause"));
        }
        self.newlines();
        match self.take() {
            Token::Interpolation {
                prefix,
                name,
                suffix,
            } if prefix.is_empty() && suffix.is_empty() && name == key => {}
            _ => return Err(Error::Unsupported("comprehension key label")),
        }
        self.expect(b':')?;
        self.newlines();
        let template = self.template(depth + 1, &key, value.as_deref())?;
        self.separators();
        if !self.symbol(b'}') {
            return Err(Error::Unsupported("comprehension yield"));
        }
        self.separators();
        if !self.symbol(b'}') {
            return Err(Error::Unsupported("sole comprehension required"));
        }
        self.document.comprehensions += 1;
        self.node(Expr::Comprehension {
            source,
            template,
            scope: self.scope,
            key,
            value,
        })
    }
    fn template(
        &mut self,
        depth: usize,
        key: &str,
        value: Option<&str>,
    ) -> Result<Template, Error> {
        if depth > 32 {
            return Err(Error::Limit("syntax depth"));
        }
        self.syntax()?;
        Ok(match self.take() {
            Token::Text(text) | Token::Multiline(text) => Template::Literal(text.into()),
            Token::Interpolation {
                prefix,
                name,
                suffix,
            } => {
                let slot = if name == key {
                    StringSlot::Key
                } else if Some(name.as_str()) == value {
                    StringSlot::Value
                } else {
                    return Err(Error::Unsupported("template interpolation slot"));
                };
                Template::Interpolated {
                    prefix,
                    slot,
                    suffix,
                }
            }
            Token::Integer(n) => Template::Literal(n.into()),
            Token::Word(word) if word == "true" || word == "false" => {
                Template::Literal((word == "true").into())
            }
            Token::Word(word) if word == key => Template::Key,
            Token::Word(word) if Some(word.as_str()) == value => Template::Value,
            Token::Symbol(b'{') => {
                let parent = self.scope;
                self.enter_scope(Some(parent));
                self.newlines();
                let mut fields = Vec::new();
                while !self.symbol(b'}') {
                    if fields.len() >= 64 {
                        return Err(Error::Limit("container items"));
                    }
                    let name = match self.take() {
                        Token::Text(name) => name,
                        Token::Word(name)
                            if !name.starts_with(['#', '_'])
                                && !matches!(
                                    name.as_str(),
                                    "let" | "for" | "if" | "in" | "import" | "package"
                                ) =>
                        {
                            if name == key || Some(name.as_str()) == value {
                                return Err(Error::Unsupported("iterator/field collision"));
                            }
                            self.scopes[self.scope].fields.insert(name.clone());
                            name
                        }
                        _ => return Err(Error::Unsupported("template field")),
                    };
                    label(&name)?;
                    if fields.iter().any(|(old, _)| old == &name) {
                        return Err(Error::Unsupported("duplicate template field"));
                    }
                    self.expect(b':')?;
                    self.newlines();
                    fields.push((name, self.template(depth + 1, key, value)?));
                    if self.symbol(b'}') {
                        break;
                    }
                    if !self.separators() {
                        return Err(Error::Unsupported("template expression"));
                    }
                }
                self.scope = parent;
                Template::Struct(fields)
            }
            Token::Symbol(b'[') => {
                self.newlines();
                let mut items = Vec::new();
                while !self.symbol(b']') {
                    if items.len() >= 64 {
                        return Err(Error::Limit("container items"));
                    }
                    items.push(self.template(depth + 1, key, value)?);
                    if self.symbol(b']') {
                        break;
                    }
                    if !self.separators() {
                        return Err(Error::Unsupported("template expression"));
                    }
                }
                Template::List(items)
            }
            _ => return Err(Error::Unsupported("template expression")),
        })
    }
    fn list(&mut self, depth: usize) -> Result<Id, Error> {
        self.newlines();
        let mut items = Vec::new();
        while !self.symbol(b']') {
            if self.peek() == &Token::Ellipsis {
                self.take();
                let tail = self.expression(depth + 1, 0)?;
                self.newlines();
                self.expect(b']')?;
                return self.node(Expr::List(items, Some(tail)));
            }
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
        }
        self.node(Expr::List(items, None))
    }
    fn file(&mut self) -> Result<String, Error> {
        self.enter_scope(None);
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
            if self.peek() == &Token::Word("let".into()) {
                self.binding(1)?;
            } else if matches!(self.peek(), Token::Word(name) if name.starts_with('#'))
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
    let marked = first
        .iter()
        .chain(&second)
        .any(|token| token == &Token::Symbol(b'*'));
    let mut parser = Parser {
        tokens: first,
        pos: 0,
        heights: Vec::new(),
        scopes: Vec::new(),
        bindings: Vec::new(),
        scope: 0,
        initializers: Vec::new(),
        document: Document {
            nodes: Vec::new(),
            definitions: BTreeMap::new(),
            roots: Vec::new(),
            marked,
            lets: 0,
            comprehensions: 0,
            projections: 0,
            concatenations: 0,
            syntax: 0,
            preparation_steps: 0,
            preparation_strings: 0,
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
    parser.document.lets = parser.bindings.len();
    bindings::resolve(&mut parser.document, &parser.scopes, &parser.bindings)?;
    Ok(parser.document)
}
