use std::collections::HashMap;

use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};
use xmltree::Element;

use super::content::decode_css_escapes;
use super::references::{
    Context, Package, ReferenceError, canonical_href, is_allowed_data_uri, normalize_path,
    parent_path, read_package, resolve_target,
};

const CSS_MEDIA_TYPE: &str = "text/css";
const KEPT_AT_RULES: [&str; 4] = ["font-face", "media", "page", "supports"];
const FORBIDDEN_PROPERTIES: [&str; 3] = ["behavior", "-moz-binding", "-ms-behavior"];
const FORBIDDEN_VALUE_TOKENS: [&str; 2] = ["expression(", "image-set("];
const FONT_DATA_URI_PREFIXES: [&str; 4] = [
    "data:font/woff2",
    "data:font/woff",
    "data:font/ttf",
    "data:font/otf",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssError {
    MissingPackage,
    InvalidOpfPath,
    InvalidPackage(ReferenceError),
}

impl std::fmt::Display for CssError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPackage => write!(f, "OPF root element is not package"),
            Self::InvalidOpfPath => write!(f, "OPF path is invalid"),
            Self::InvalidPackage(error) => write!(f, "package cannot be read: {error}"),
        }
    }
}

impl std::error::Error for CssError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CssEdits {
    pub removed_at_rules: usize,
    pub removed_rules: usize,
    pub removed_declarations: usize,
    pub removed_font_sources: usize,
    pub rewritten_urls: usize,
    pub repaired_syntax: usize,
}

impl CssEdits {
    fn any(&self) -> bool {
        self.removed_at_rules
            + self.removed_rules
            + self.removed_declarations
            + self.removed_font_sources
            + self.rewritten_urls
            + self.repaired_syntax
            > 0
    }

    fn add(&mut self, other: &CssEdits) {
        self.removed_at_rules += other.removed_at_rules;
        self.removed_rules += other.removed_rules;
        self.removed_declarations += other.removed_declarations;
        self.removed_font_sources += other.removed_font_sources;
        self.rewritten_urls += other.rewritten_urls;
        self.repaired_syntax += other.repaired_syntax;
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CssReport {
    pub changed: bool,
    pub transcoded_stylesheets: Vec<String>,
    pub edits: CssEdits,
}

enum Terminator {
    Block,
    Semicolon,
    CloseBrace,
    End,
}

enum Item<'a> {
    Rule {
        prelude: &'a str,
        block: &'a str,
    },
    AtRule {
        name: &'a str,
        prelude: &'a str,
        block: Option<&'a str>,
    },
}

struct UrlCall {
    start: usize,
    end: usize,
    target: String,
}

struct Scanner<'a> {
    text: &'a str,
    position: usize,
}

impl<'a> Scanner<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, position: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.position).copied()
    }

    fn advance(&mut self, count: usize) {
        self.position = (self.position + count).min(self.text.len());
    }

    fn starts_with(&self, prefix: &[u8]) -> bool {
        self.text.as_bytes()[self.position..].starts_with(prefix)
    }

    fn skip_ascii_whitespace(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.advance(1);
        }
    }

    fn skip_trivia(&mut self) {
        loop {
            if self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
                self.advance(1);
            } else if self.starts_with(b"<!--") {
                self.advance(4);
            } else if self.starts_with(b"-->") {
                self.advance(3);
            } else {
                return;
            }
        }
    }

    fn skip_string(&mut self) {
        let Some(quote) = self.peek() else {
            return;
        };
        self.advance(1);

        while let Some(byte) = self.peek() {
            match byte {
                b'\\' => self.advance(2),
                b'\n' => return,
                byte if byte == quote => {
                    self.advance(1);
                    return;
                }
                _ => self.advance(1),
            }
        }
    }

    fn skip_group(&mut self) {
        let mut depth = 0usize;

        while let Some(byte) = self.peek() {
            match byte {
                b'"' | b'\'' => self.skip_string(),
                b'\\' => self.advance(2),
                b'(' | b'[' => {
                    depth += 1;
                    self.advance(1);
                }
                b')' | b']' => {
                    depth = depth.saturating_sub(1);
                    self.advance(1);
                    if depth == 0 {
                        return;
                    }
                }
                _ => self.advance(1),
            }
        }
    }

    fn read_prelude(&mut self) -> (&'a str, Terminator) {
        let start = self.position;

        loop {
            let terminator = match self.peek() {
                None => Terminator::End,
                Some(b'"' | b'\'') => {
                    self.skip_string();
                    continue;
                }
                Some(b'(' | b'[') => {
                    self.skip_group();
                    continue;
                }
                Some(b'\\') => {
                    self.advance(2);
                    continue;
                }
                Some(b'{') => Terminator::Block,
                Some(b';') => Terminator::Semicolon,
                Some(b'}') => Terminator::CloseBrace,
                Some(_) => {
                    self.advance(1);
                    continue;
                }
            };
            return (&self.text[start..self.position], terminator);
        }
    }

    /// Returns the text inside the block and whether its closing brace was found.
    fn read_block(&mut self) -> (&'a str, bool) {
        self.advance(1);
        let start = self.position;
        let mut depth = 1usize;

        while let Some(byte) = self.peek() {
            match byte {
                b'"' | b'\'' => self.skip_string(),
                b'(' | b'[' => self.skip_group(),
                b'\\' => self.advance(2),
                b'{' => {
                    depth += 1;
                    self.advance(1);
                }
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        let inner = &self.text[start..self.position];
                        self.advance(1);
                        return (inner, true);
                    }
                    self.advance(1);
                }
                _ => self.advance(1),
            }
        }

        (&self.text[start..], false)
    }

    fn read_rule(&mut self, edits: &mut CssEdits) -> Option<Item<'a>> {
        let (prelude, terminator) = self.read_prelude();

        match terminator {
            Terminator::Block => {
                let (block, closed) = self.read_block();
                edits.repaired_syntax += usize::from(!closed);
                Some(Item::Rule { prelude, block })
            }
            Terminator::Semicolon => {
                self.advance(1);
                edits.repaired_syntax += 1;
                None
            }
            Terminator::CloseBrace | Terminator::End => {
                edits.repaired_syntax += 1;
                None
            }
        }
    }

    fn read_at_rule(&mut self, edits: &mut CssEdits) -> Item<'a> {
        self.advance(1);
        let name_start = self.position;
        while self.peek().is_some_and(is_identifier_byte) {
            self.advance(1);
        }
        let name = &self.text[name_start..self.position];
        let (prelude, terminator) = self.read_prelude();

        let block = match terminator {
            Terminator::Block => {
                let (block, closed) = self.read_block();
                edits.repaired_syntax += usize::from(!closed);
                Some(block)
            }
            Terminator::Semicolon => {
                self.advance(1);
                None
            }
            Terminator::CloseBrace | Terminator::End => None,
        };

        Item::AtRule {
            name,
            prelude,
            block,
        }
    }

    fn at_url_function(&self) -> bool {
        let bytes = self.text.as_bytes();
        let rest = &bytes[self.position..];
        let preceded_by_identifier =
            self.position > 0 && is_identifier_byte(bytes[self.position - 1]);

        rest.len() >= 4 && rest[..4].eq_ignore_ascii_case(b"url(") && !preceded_by_identifier
    }

    fn read_url_call(&mut self) -> UrlCall {
        let start = self.position;
        self.advance(4);
        self.skip_ascii_whitespace();

        let raw = match self.peek() {
            Some(quote @ (b'"' | b'\'')) => self.read_quoted(quote),
            _ => self.read_unquoted(),
        };

        self.skip_ascii_whitespace();
        let closed = self.peek() == Some(b')');
        self.advance(usize::from(closed));
        let target = if closed {
            decode_css_escapes(raw.trim())
        } else {
            String::new()
        };

        UrlCall {
            start,
            end: self.position,
            target,
        }
    }

    fn read_quoted(&mut self, quote: u8) -> &'a str {
        let content_start = self.position + 1;
        self.skip_string();
        let closed = self.position > content_start
            && self.text.as_bytes()[self.position - 1] == quote;
        let content_end = if closed {
            self.position - 1
        } else {
            self.position
        };

        &self.text[content_start..content_end]
    }

    fn read_unquoted(&mut self) -> &'a str {
        let start = self.position;
        while self.peek().is_some_and(|byte| byte != b')') {
            self.advance(1);
        }

        &self.text[start..self.position]
    }
}

/// Makes every stylesheet in the manifest safe and self-contained (Y1-Y4).
///
/// Only `@font-face`, `@media`, `@page` and `@supports` survive. Declarations
/// that load code, remote resources or anything that is not a manifest image or
/// font are dropped, and the urls that remain are rewritten to the same
/// canonical form as content references. Selectors are not scoped (Y5): the
/// reader contains the stylesheet in a shadow root, so the book's CSS cannot
/// reach the app. A stylesheet with no edits keeps its exact bytes.
pub fn repair_css(
    opf: &Element,
    opf_path: &str,
    archive: &mut HashMap<String, Vec<u8>>,
) -> Result<CssReport, CssError> {
    if opf.name != "package" {
        return Err(CssError::MissingPackage);
    }

    let opf_path = normalize_path(opf_path).ok_or(CssError::InvalidOpfPath)?;
    let package = read_package(opf, &parent_path(&opf_path)).map_err(CssError::InvalidPackage)?;
    let mut report = CssReport::default();

    let mut stylesheets: Vec<&String> = package
        .media_types
        .iter()
        .filter(|(path, media_type)| {
            media_type.as_str() == CSS_MEDIA_TYPE && archive.contains_key(*path)
        })
        .map(|(path, _)| path)
        .collect();
    stylesheets.sort();

    for path in stylesheets {
        repair_stylesheet(path, archive, &package, &mut report);
    }

    Ok(report)
}

fn repair_stylesheet(
    path: &str,
    archive: &mut HashMap<String, Vec<u8>>,
    package: &Package,
    report: &mut CssReport,
) {
    let Some(bytes) = archive.get(path) else {
        return;
    };
    let (text, transcoded) = decode_css(bytes);
    let directory = parent_path(path);
    let context = Context {
        document_dir: &directory,
        package,
    };
    let mut edits = CssEdits::default();

    let sanitized = sanitize_stylesheet(&text, &context, &mut edits);

    report.edits.add(&edits);
    if transcoded {
        report.transcoded_stylesheets.push(path.to_owned());
    }
    if transcoded || edits.any() {
        archive.insert(path.to_owned(), sanitized.into_bytes());
        report.changed = true;
    }
}

/// Returns the text and whether it differed from valid UTF-8 without a BOM.
///
/// Valid UTF-8 is always read as UTF-8; `@charset` is only consulted for
/// bytes that are not.
fn decode_css(bytes: &[u8]) -> (String, bool) {
    if let Some((encoding, bom_length)) = Encoding::for_bom(bytes) {
        let (text, _) = encoding.decode_without_bom_handling(&bytes[bom_length..]);
        return (text.into_owned(), true);
    }

    match std::str::from_utf8(bytes) {
        Ok(text) => (text.to_owned(), false),
        Err(_) => {
            let encoding = declared_charset(bytes).unwrap_or(WINDOWS_1252);
            let (text, _) = encoding.decode_without_bom_handling(bytes);
            (text.into_owned(), true)
        }
    }
}

fn declared_charset(bytes: &[u8]) -> Option<&'static Encoding> {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(100)]);
    let label = head.strip_prefix("@charset \"")?.split('"').next()?;

    Encoding::for_label(label.as_bytes()).filter(|encoding| *encoding != UTF_8)
}

fn sanitize_stylesheet(text: &str, context: &Context, edits: &mut CssEdits) -> String {
    let source = strip_comments(text, edits);
    let rules = sanitize_items(parse_items(&source, edits), context, edits);

    if rules.is_empty() {
        String::new()
    } else {
        format!("{}\n", rules.join("\n"))
    }
}

fn strip_comments(text: &str, edits: &mut CssEdits) -> String {
    let mut stripped = String::with_capacity(text.len());
    let mut scanner = Scanner::new(text);
    let mut copied_up_to = 0;

    while let Some(byte) = scanner.peek() {
        if matches!(byte, b'"' | b'\'') {
            scanner.skip_string();
        } else if scanner.starts_with(b"/*") {
            stripped.push_str(&text[copied_up_to..scanner.position]);
            stripped.push(' ');
            match text[scanner.position + 2..].find("*/") {
                Some(offset) => scanner.advance(offset + 4),
                None => {
                    edits.repaired_syntax += 1;
                    scanner.advance(text.len());
                }
            }
            copied_up_to = scanner.position;
        } else {
            scanner.advance(1);
        }
    }

    stripped.push_str(&text[copied_up_to..]);
    stripped
}

fn parse_items<'a>(text: &'a str, edits: &mut CssEdits) -> Vec<Item<'a>> {
    let mut scanner = Scanner::new(text);
    let mut items = Vec::new();

    loop {
        scanner.skip_trivia();
        match scanner.peek() {
            None => return items,
            Some(b'}') => {
                scanner.advance(1);
                edits.repaired_syntax += 1;
            }
            Some(b'@') => items.push(scanner.read_at_rule(edits)),
            Some(_) => items.extend(scanner.read_rule(edits)),
        }
    }
}

fn sanitize_items(items: Vec<Item>, context: &Context, edits: &mut CssEdits) -> Vec<String> {
    items
        .into_iter()
        .filter_map(|item| match item {
            Item::Rule { prelude, block } => sanitize_rule(prelude, block, context, edits),
            Item::AtRule {
                name,
                prelude,
                block,
            } => sanitize_at_rule(name, prelude, block, context, edits),
        })
        .collect()
}

fn sanitize_rule(
    prelude: &str,
    block: &str,
    context: &Context,
    edits: &mut CssEdits,
) -> Option<String> {
    let selector = prelude.trim();
    let declarations = sanitize_declarations(block, false, context, edits);

    if selector.is_empty() || is_forbidden_value(selector) || declarations.is_empty() {
        edits.removed_rules += 1;
        return None;
    }

    Some(format!("{selector}{{{}}}", declarations.join(";")))
}

fn sanitize_at_rule(
    name: &str,
    prelude: &str,
    block: Option<&str>,
    context: &Context,
    edits: &mut CssEdits,
) -> Option<String> {
    let name = name.to_ascii_lowercase();
    let prelude = prelude.trim();

    let body = match block {
        Some(block) if KEPT_AT_RULES.contains(&name.as_str()) && !is_forbidden_value(prelude) => {
            sanitize_at_rule_body(&name, block, context, edits)
        }
        _ => String::new(),
    };

    if body.is_empty() {
        edits.removed_at_rules += 1;
        return None;
    }

    let header = if prelude.is_empty() {
        format!("@{name}")
    } else {
        format!("@{name} {prelude}")
    };
    Some(format!("{header}{{{body}}}"))
}

fn sanitize_at_rule_body(
    name: &str,
    block: &str,
    context: &Context,
    edits: &mut CssEdits,
) -> String {
    if matches!(name, "media" | "supports") {
        return sanitize_items(parse_items(block, edits), context, edits).join("\n");
    }

    let is_font_face = name == "font-face";
    let declarations = sanitize_declarations(block, is_font_face, context, edits);
    let has_source = declarations
        .iter()
        .any(|declaration| declaration.starts_with("src:"));

    if is_font_face && !has_source {
        return String::new();
    }
    declarations.join(";")
}

fn sanitize_declarations(
    block: &str,
    is_font_face: bool,
    context: &Context,
    edits: &mut CssEdits,
) -> Vec<String> {
    let mut scanner = Scanner::new(block);
    let mut declarations = Vec::new();

    loop {
        scanner.skip_trivia();
        if scanner.peek().is_none() {
            return declarations;
        }

        let (chunk, terminator) = scanner.read_prelude();
        match terminator {
            Terminator::Block => {
                scanner.read_block();
                edits.removed_rules += 1;
            }
            Terminator::Semicolon => {
                scanner.advance(1);
                declarations.extend(sanitize_declaration(chunk, is_font_face, context, edits));
            }
            Terminator::CloseBrace => {
                scanner.advance(1);
                edits.repaired_syntax += 1;
            }
            Terminator::End => {
                declarations.extend(sanitize_declaration(chunk, is_font_face, context, edits));
            }
        }
    }
}

fn sanitize_declaration(
    chunk: &str,
    is_font_face: bool,
    context: &Context,
    edits: &mut CssEdits,
) -> Option<String> {
    let chunk = chunk.trim();
    if chunk.is_empty() {
        return None;
    }

    let Some((name, value)) = split_declaration(chunk) else {
        edits.removed_declarations += 1;
        return None;
    };

    let is_unsafe = is_forbidden_property(&name)
        || is_forbidden_value(value)
        || hides_url_function(chunk);
    let rewritten = if is_unsafe {
        None
    } else if is_font_face && name == "src" {
        rewrite_font_sources(value, context, edits)
    } else {
        rewrite_urls(value, context, edits)
    };

    match rewritten {
        Some(value) if !value.trim().is_empty() => Some(format!("{name}:{value}")),
        _ => {
            edits.removed_declarations += 1;
            None
        }
    }
}

fn split_declaration(chunk: &str) -> Option<(String, &str)> {
    let (name, value) = chunk.split_once(':')?;
    let name = name.trim();
    let value = value.trim();
    let is_valid_name = !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'));

    if !is_valid_name || value.is_empty() {
        return None;
    }

    let name = if name.starts_with("--") {
        name.to_owned()
    } else {
        name.to_ascii_lowercase()
    };
    Some((name, value))
}

fn rewrite_font_sources(value: &str, context: &Context, edits: &mut CssEdits) -> Option<String> {
    let kept: Vec<String> = split_top_level_commas(value)
        .into_iter()
        .filter_map(|source| {
            let rewritten = rewrite_urls(source, context, edits);
            edits.removed_font_sources += usize::from(rewritten.is_none());
            rewritten
        })
        .collect();

    (!kept.is_empty()).then(|| kept.join(", "))
}

fn split_top_level_commas(value: &str) -> Vec<&str> {
    let mut scanner = Scanner::new(value);
    let mut parts = Vec::new();
    let mut start = 0;

    while let Some(byte) = scanner.peek() {
        match byte {
            b'"' | b'\'' => scanner.skip_string(),
            b'(' | b'[' => scanner.skip_group(),
            b',' => {
                parts.push(value[start..scanner.position].trim());
                scanner.advance(1);
                start = scanner.position;
            }
            _ => scanner.advance(1),
        }
    }

    parts.push(value[start..].trim());
    parts
}

/// Returns `None` when any url in the value cannot be kept.
fn rewrite_urls(value: &str, context: &Context, edits: &mut CssEdits) -> Option<String> {
    let mut rewritten = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(call) = find_url_call(rest) {
        rewritten.push_str(&rest[..call.start]);
        let canonical = canonical_target(&call.target, context)?;

        if canonical == call.target {
            rewritten.push_str(&rest[call.start..call.end]);
        } else {
            rewritten.push_str(&format!("url(\"{canonical}\")"));
            edits.rewritten_urls += 1;
        }
        rest = &rest[call.end..];
    }

    rewritten.push_str(rest);
    Some(rewritten)
}

fn find_url_call(text: &str) -> Option<UrlCall> {
    let mut scanner = Scanner::new(text);

    while let Some(byte) = scanner.peek() {
        if matches!(byte, b'"' | b'\'') {
            scanner.skip_string();
        } else if scanner.at_url_function() {
            return Some(scanner.read_url_call());
        } else {
            scanner.advance(1);
        }
    }

    None
}

fn canonical_target(target: &str, context: &Context) -> Option<String> {
    if target.starts_with('#') || is_allowed_data_uri(target) || is_font_data_uri(target) {
        return (!target.contains(['"', '\\'])).then(|| target.to_owned());
    }

    let (path, suffix) = resolve_target(target, context)?;
    let media_type = context.package.media_types.get(&path)?;
    let is_resource = media_type.starts_with("image/") || media_type.starts_with("font/");

    is_resource.then(|| canonical_href(context.document_dir, &path, &suffix))
}

fn is_font_data_uri(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    FONT_DATA_URI_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}

fn is_forbidden_property(name: &str) -> bool {
    FORBIDDEN_PROPERTIES.contains(&normalize(name).as_str())
}

fn is_forbidden_value(text: &str) -> bool {
    let normalized = normalize(text);
    FORBIDDEN_VALUE_TOKENS
        .iter()
        .any(|token| normalized.contains(token))
}

/// A `url(` spelled with CSS escapes (`\75rl(`) is still a url function to the
/// browser. Such a declaration cannot be rewritten reliably, so it is dropped.
fn hides_url_function(declaration: &str) -> bool {
    let literal = declaration
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();

    normalize(declaration).matches("url(").count() != literal.matches("url(").count()
}

fn normalize(text: &str) -> String {
    decode_css_escapes(text)
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
}

#[cfg(test)]
#[path = "css_tests.rs"]
mod css_tests;