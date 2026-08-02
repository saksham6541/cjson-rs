use crate::{error::ParseError, Value};

const NESTING_LIMIT: usize = 1000;

pub fn parse(input: &str) -> Result<Value, ParseError> {
    let mut parser = Parser::new(input);
    // Match cJSON_Parse: parse one JSON value and stop. Trailing content after
    // the first complete value is ignored (not an error). See DECISIONS.md
    // [Task 4] — confirmed by differential fuzz, then fixed to match upstream.
    let value = parser.parse_value(0)?;
    Ok(value)
}

struct Parser<'a> {
    chars: Vec<char>,
    // byte_offsets[i] is the byte offset of chars[i] in the original input;
    // byte_offsets[chars.len()] is the total byte length. Needed because
    // cJSON's `cJSON_GetErrorPtr` reports a *byte* offset into the raw
    // buffer, but we walk the input as `char`s (see §5: "ideally the same
    // error position"). Without this, any error after a multibyte UTF-8
    // character would report the wrong position relative to the C reference.
    // See DECISIONS.md.
    byte_offsets: Vec<usize>,
    position: usize,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        let mut chars = Vec::new();
        let mut byte_offsets = Vec::new();
        for (byte_idx, ch) in input.char_indices() {
            byte_offsets.push(byte_idx);
            chars.push(ch);
        }
        byte_offsets.push(input.len());
        Self {
            chars,
            byte_offsets,
            position: 0,
            _marker: std::marker::PhantomData,
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<Value, ParseError> {
        self.skip_whitespace();
        if depth >= NESTING_LIMIT {
            return Err(ParseError::new("nesting limit exceeded", self.position()));
        }

        match self.peek() {
            Some('{') => self.parse_object(depth + 1),
            Some('[') => self.parse_array(depth + 1),
            Some('"') => self.parse_string().map(Value::String),
            Some('t') => self.parse_literal("true", Value::Bool(true)),
            Some('f') => self.parse_literal("false", Value::Bool(false)),
            Some('n') => self.parse_literal("null", Value::Null),
            Some('-') | Some('0'..='9') => self.parse_number(),
            Some(ch) => Err(ParseError::new(
                format!("unexpected token {ch}"),
                self.position(),
            )),
            None => Err(ParseError::new("unexpected end of input", self.position())),
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<Value, ParseError> {
        self.expect('{')?;
        self.skip_whitespace();
        let mut entries = Vec::new();
        if self.consume_if('}') {
            return Ok(Value::Object(entries));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect(':')?;
            let value = self.parse_value(depth)?;
            entries.push((key, value));
            self.skip_whitespace();
            if self.consume_if('}') {
                break;
            }
            self.expect(',')?;
        }

        Ok(Value::Object(entries))
    }

    fn parse_array(&mut self, depth: usize) -> Result<Value, ParseError> {
        self.expect('[')?;
        self.skip_whitespace();
        let mut items = Vec::new();
        if self.consume_if(']') {
            return Ok(Value::Array(items));
        }

        loop {
            items.push(self.parse_value(depth)?);
            self.skip_whitespace();
            if self.consume_if(']') {
                break;
            }
            self.expect(',')?;
        }

        Ok(Value::Array(items))
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.expect('"')?;
        let mut output = String::new();
        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.advance();
                    return Ok(output);
                }
                '\\' => {
                    self.advance();
                    let escaped = self
                        .peek()
                        .ok_or_else(|| ParseError::new("unterminated escape", self.position()))?;
                    match escaped {
                        '"' => output.push('"'),
                        '\\' => output.push('\\'),
                        '/' => output.push('/'),
                        'b' => output.push('\u{0008}'),
                        'f' => output.push('\u{000C}'),
                        'n' => output.push('\n'),
                        'r' => output.push('\r'),
                        't' => output.push('\t'),
                        'u' => {
                            self.advance();
                            let codepoint = self.read_unicode_escape()?;
                            // Deliberate deviation from cJSON (see §5 and
                            // DECISIONS.md): cJSON's C strings are
                            // null-terminated, so `\u0000` can't survive
                            // round-trip and is effectively unrepresentable.
                            // Rust's `String` has no such limitation, so
                            // `\u0000` (codepoint 0) is accepted here and
                            // will round-trip through the printer's
                            // `\u0000` escaping. Raw unescaped control
                            // bytes (including NUL) are also accepted to
                            // match cJSON — see DECISIONS.md [Task 4].
                            output.push(char::from_u32(codepoint).ok_or_else(|| {
                                ParseError::new("invalid unicode escape", self.position())
                            })?);
                        }
                        _ => {
                            return Err(ParseError::new(
                                "invalid escape sequence",
                                self.position(),
                            ));
                        }
                    }
                    self.advance();
                }
                // Match cJSON: raw C0 control characters (0x00–0x1F) inside
                // string literals are accepted and re-escaped on print
                // (printer emits \u00XX). Confirmed divergence from fuzz;
                // fixed to match upstream — see DECISIONS.md [Task 4].
                _ => {
                    output.push(ch);
                    self.advance();
                }
            }
        }

        Err(ParseError::new("unterminated string", self.position()))
    }

    fn parse_number(&mut self) -> Result<Value, ParseError> {
        let start = self.position();
        let mut buffer = String::new();
        if self.consume_if('-') {
            buffer.push('-');
        }

        if self.consume_if('0') {
            buffer.push('0');
        } else if let Some('1'..='9') = self.peek() {
            while let Some(ch @ '0'..='9') = self.peek() {
                buffer.push(ch);
                self.advance();
            }
        } else {
            return Err(ParseError::new("invalid number", start));
        }

        if self.consume_if('.') {
            buffer.push('.');
            if self.peek().is_none() || self.peek().filter(|ch| ch.is_ascii_digit()).is_none() {
                return Err(ParseError::new("invalid number", self.position()));
            }
            while let Some(ch @ '0'..='9') = self.peek() {
                buffer.push(ch);
                self.advance();
            }
        }

        if self.consume_if('e') || self.consume_if('E') {
            buffer.push('e');
            if self.consume_if('+') {
                buffer.push('+');
            } else if self.consume_if('-') {
                buffer.push('-');
            }
            if self.peek().is_none() || self.peek().filter(|ch| ch.is_ascii_digit()).is_none() {
                return Err(ParseError::new("invalid exponent", self.position()));
            }
            while let Some(ch @ '0'..='9') = self.peek() {
                buffer.push(ch);
                self.advance();
            }
        }

        let parsed = buffer
            .parse::<f64>()
            .map_err(|_| ParseError::new("invalid number", start))?;
        Ok(Value::Number(parsed))
    }

    fn parse_literal(&mut self, expected: &str, value: Value) -> Result<Value, ParseError> {
        let start = self.position;
        let chars: Vec<char> = expected.chars().collect();
        let mut matches = true;
        for (offset, ch) in chars.iter().enumerate() {
            if self.chars.get(start + offset).copied() != Some(*ch) {
                matches = false;
                break;
            }
        }

        if matches {
            self.position += chars.len();
            Ok(value)
        } else {
            Err(ParseError::new(
                format!("expected {expected}"),
                self.position(),
            ))
        }
    }

    fn read_unicode_escape(&mut self) -> Result<u32, ParseError> {
        let mut code = 0u32;
        for _ in 0..4 {
            let ch = self
                .peek()
                .ok_or_else(|| ParseError::new("unterminated unicode escape", self.position()))?;
            let digit = ch
                .to_digit(16)
                .ok_or_else(|| ParseError::new("invalid unicode escape", self.position()))?;
            code = (code << 4) | digit;
            self.advance();
        }
        Ok(code)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => self.advance(),
                _ => break,
            }
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), ParseError> {
        if self.consume_if(expected) {
            Ok(())
        } else {
            Err(ParseError::new(
                format!("expected {expected}"),
                self.position(),
            ))
        }
    }

    fn consume_if(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) {
        if self.position < self.chars.len() {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }


    /// Byte offset of the current position in the original input, matching
    /// cJSON's `cJSON_GetErrorPtr` semantics (a raw byte pointer), not a
    /// char index.
    fn position(&self) -> usize {
        self.byte_offsets[self.position]
    }
}
