//! JSON duplicate-key pre-scan (spec §5.2 / §3.1).
//!
//! Mainstream parsers such as serde_json silently keep the last value of a
//! duplicate key, but the protocol pins duplicate keys as an error (node-level
//! E-META-SYNTAX / document-level W-DOC-META), so a streaming pre-scan runs
//! before serde.
//!
//! Semantics: Ok(()) means the input is valid JSON with no in-object duplicate
//! keys; Err(()) means a duplicate key or malformed JSON (the latter is also
//! rejected by serde_json, so both paths converge on the pinned failure).

#[allow(clippy::result_unit_err)]
pub fn scan(body: &str) -> Result<(), ()> {
    let chars: Vec<char> = body.chars().collect();
    let mut p = P { c: &chars, i: 0 };
    p.value()?;
    p.ws();
    if p.i != p.c.len() {
        return Err(());
    }
    Ok(())
}

pub fn duplicate_keys(body: &str) -> bool {
    scan(body).is_err()
}

struct P<'a> {
    c: &'a [char],
    i: usize,
}

impl<'a> P<'a> {
    fn peek(&self) -> Option<char> {
        self.c.get(self.i).copied()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.i += 1;
        }
        ch
    }

    fn ws(&mut self) {
        while matches!(self.peek(), Some(' ') | Some('\t') | Some('\n') | Some('\r')) {
            self.i += 1;
        }
    }

    fn lit(&mut self, word: &str) -> Result<(), ()> {
        for expect in word.chars() {
            if self.next() != Some(expect) {
                return Err(());
            }
        }
        Ok(())
    }

    fn string(&mut self) -> Result<Vec<char>, ()> {
        if self.next() != Some('"') {
            return Err(());
        }
        let mut out = Vec::new();
        loop {
            match self.next() {
                None => return Err(()),
                Some('"') => return Ok(out),
                Some('\\') => match self.next() {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('/') => out.push('/'),
                    Some('b') => out.push('\u{0008}'),
                    Some('f') => out.push('\u{000C}'),
                    Some('n') => out.push('\n'),
                    Some('r') => out.push('\r'),
                    Some('t') => out.push('\t'),
                    Some('u') => {
                        for _ in 0..4 {
                            match self.next() {
                                Some(h) if h.is_ascii_hexdigit() => {}
                                _ => return Err(()),
                            }
                        }
                    }
                    _ => return Err(()),
                },
                Some(ch) => out.push(ch),
            }
        }
    }

    fn number(&mut self) -> Result<(), ()> {
        let start = self.i;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || matches!(ch, '-' | '+' | '.' | 'e' | 'E') {
                self.i += 1;
            } else {
                break;
            }
        }
        if self.i == start {
            return Err(());
        }
        Ok(())
    }

    fn value(&mut self) -> Result<(), ()> {
        self.ws();
        match self.peek() {
            Some('{') => {
                self.i += 1;
                let mut keys: Vec<Vec<char>> = Vec::new();
                self.ws();
                if self.peek() == Some('}') {
                    self.i += 1;
                    return Ok(());
                }
                loop {
                    self.ws();
                    let key = self.string()?;
                    if keys.contains(&key) {
                        return Err(());
                    }
                    keys.push(key);
                    self.ws();
                    if self.next() != Some(':') {
                        return Err(());
                    }
                    self.value()?;
                    self.ws();
                    match self.next() {
                        Some(',') => continue,
                        Some('}') => return Ok(()),
                        _ => return Err(()),
                    }
                }
            }
            Some('[') => {
                self.i += 1;
                self.ws();
                if self.peek() == Some(']') {
                    self.i += 1;
                    return Ok(());
                }
                loop {
                    self.value()?;
                    self.ws();
                    match self.next() {
                        Some(',') => continue,
                        Some(']') => return Ok(()),
                        _ => return Err(()),
                    }
                }
            }
            Some('"') => self.string().map(|_| ()),
            Some('t') => self.lit("true"),
            Some('f') => self.lit("false"),
            Some('n') => self.lit("null"),
            Some(_) => self.number(),
            None => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_duplicate_keys() {
        assert!(duplicate_keys(r#"{"a":1,"a":2}"#));
        assert!(duplicate_keys(r#"{"a":{"b":1,"b":2}}"#));
        assert!(!duplicate_keys(r#"{"a":1,"b":2}"#));
        assert!(!duplicate_keys(r#"{"a":"x}x","b":[1,2]}"#));
    }

    #[test]
    fn malformed_is_err() {
        assert!(scan("{").is_err());
        assert!(scan("{} trailing").is_err());
        assert!(scan(r#"{"a":}"#).is_err());
        assert!(scan("{}").is_ok());
        assert!(scan("[]").is_ok());
        assert!(scan("null").is_ok());
    }
}
