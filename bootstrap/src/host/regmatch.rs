use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Rm2Error {
    PatternTooLong { len: usize },
    InvalidPattern,
}

impl std::fmt::Display for Rm2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rm2Error::PatternTooLong { len } => write!(f, "pattern too long ({len})"),
            Rm2Error::InvalidPattern => write!(f, "invalid pattern"),
        }
    }
}

impl std::error::Error for Rm2Error {}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Literal(u8),
    Dot,
    Star(Box<Token>),
    Plus(Box<Token>),
    Question(Box<Token>),
    CharClass(Vec<u8>),
    NegClass(Vec<u8>),
}

pub struct RegMatch {
    patterns: BTreeMap<u64, Vec<Token>>,
    total_compiles: u64,
    total_matches: u64,
    total_hits: u64,
    total_misses: u64,
}

impl RegMatch {
    pub fn new() -> Self { Self { patterns: BTreeMap::new(), total_compiles: 0, total_matches: 0, total_hits: 0, total_misses: 0 } }

    pub fn compile(&mut self, id: u64, pattern: &[u8]) -> Result<(), Rm2Error> {
        if pattern.len() > 256 { return Err(Rm2Error::PatternTooLong { len: pattern.len() }); }
        let tokens = Self::parse(pattern)?;
        self.patterns.insert(id, tokens);
        self.total_compiles += 1;
        Ok(())
    }

    fn parse(pattern: &[u8]) -> Result<Vec<Token>, Rm2Error> {
        let mut tokens = Vec::new();
        let mut i = 0;
        while i < pattern.len() {
            match pattern[i] {
                b'.' => { tokens.push(Token::Dot); i += 1; }
                b'[' => {
                    let neg = i + 1 < pattern.len() && pattern[i + 1] == b'^';
                    let start = if neg { i + 2 } else { i + 1 };
                    let mut end = start;
                    while end < pattern.len() && pattern[end] != b']' { end += 1; }
                    if end >= pattern.len() { return Err(Rm2Error::InvalidPattern); }
                    let chars: Vec<u8> = pattern[start..end].to_vec();
                    if neg { tokens.push(Token::NegClass(chars)); } else { tokens.push(Token::CharClass(chars)); }
                    i = end + 1;
                }
                b'*' => {
                    if let Some(prev) = tokens.pop() {
                        tokens.push(Token::Star(Box::new(prev)));
                    }
                    i += 1;
                }
                b'+' => {
                    if let Some(prev) = tokens.pop() {
                        tokens.push(Token::Plus(Box::new(prev)));
                    }
                    i += 1;
                }
                b'?' => {
                    if let Some(prev) = tokens.pop() {
                        tokens.push(Token::Question(Box::new(prev)));
                    }
                    i += 1;
                }
                c => { tokens.push(Token::Literal(c)); i += 1; }
            }
        }
        Ok(tokens)
    }

    pub fn is_match(&mut self, id: u64, input: &[u8]) -> bool {
        self.total_matches += 1;
        if let Some(tokens) = self.patterns.get(&id) {
            let result = Self::match_tokens(tokens, input, 0, 0);
            if result { self.total_hits += 1; } else { self.total_misses += 1; }
            result
        } else { false }
    }

    fn match_tokens(tokens: &[Token], input: &[u8], ti: usize, ii: usize) -> bool {
        if ti >= tokens.len() { return ii >= input.len() || ti == tokens.len(); }
        if ii >= input.len() {
            return tokens[ti..].iter().all(|t| matches!(t, Token::Star(_) | Token::Question(_)));
        }
        match &tokens[ti] {
            Token::Literal(c) => {
                if ii < input.len() && input[ii] == *c {
                    Self::match_tokens(tokens, input, ti + 1, ii + 1)
                } else { false }
            }
            Token::Dot => Self::match_tokens(tokens, input, ti + 1, ii + 1),
            Token::Star(inner) => {
                if Self::match_tokens(tokens, input, ti + 1, ii) { return true; }
                if Self::token_matches(inner, input, ii) {
                    Self::match_tokens(tokens, input, ti, ii + 1)
                } else { Self::match_tokens(tokens, input, ti + 1, ii) }
            }
            Token::Plus(inner) => {
                if Self::token_matches(inner, input, ii) {
                    Self::match_tokens(tokens, input, ti, ii + 1) || Self::match_tokens(tokens, input, ti + 1, ii + 1)
                } else { false }
            }
            Token::Question(inner) => {
                if Self::token_matches(inner, input, ii) && Self::match_tokens(tokens, input, ti + 1, ii + 1) { return true; }
                Self::match_tokens(tokens, input, ti + 1, ii)
            }
            Token::CharClass(chars) => {
                if ii < input.len() && chars.contains(&input[ii]) {
                    Self::match_tokens(tokens, input, ti + 1, ii + 1)
                } else { false }
            }
            Token::NegClass(chars) => {
                if ii < input.len() && !chars.contains(&input[ii]) {
                    Self::match_tokens(tokens, input, ti + 1, ii + 1)
                } else { false }
            }
        }
    }

    fn token_matches(token: &Token, input: &[u8], idx: usize) -> bool {
        if idx >= input.len() { return false; }
        match token {
            Token::Literal(c) => input[idx] == *c,
            Token::Dot => true,
            Token::CharClass(chars) => chars.contains(&input[idx]),
            Token::NegClass(chars) => !chars.contains(&input[idx]),
            _ => false,
        }
    }

    pub fn pattern_count(&self) -> usize { self.patterns.len() }
    pub fn total_compiles(&self) -> u64 { self.total_compiles }
    pub fn total_matches(&self) -> u64 { self.total_matches }
    pub fn total_hits(&self) -> u64 { self.total_hits }
    pub fn total_misses(&self) -> u64 { self.total_misses }
}

impl Default for RegMatch {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_matcher() { let rm = RegMatch::new(); assert_eq!(rm.pattern_count(), 0); }

    #[test]
    fn literal_match() {
        let mut rm = RegMatch::new();
        rm.compile(1, b"hello").unwrap();
        assert!(rm.is_match(1, b"hello"));
        assert!(!rm.is_match(1, b"world"));
    }

    #[test]
    fn dot_match() {
        let mut rm = RegMatch::new();
        rm.compile(1, b"h.llo").unwrap();
        assert!(rm.is_match(1, b"hello"));
        assert!(rm.is_match(1, b"hallo"));
    }

    #[test]
    fn star_match() {
        let mut rm = RegMatch::new();
        rm.compile(1, b"ab*c").unwrap();
        assert!(rm.is_match(1, b"ac"));
        assert!(rm.is_match(1, b"abc"));
        assert!(rm.is_match(1, b"abbc"));
    }

    #[test]
    fn plus_match() {
        let mut rm = RegMatch::new();
        rm.compile(1, b"ab+c").unwrap();
        assert!(!rm.is_match(1, b"ac"));
        assert!(rm.is_match(1, b"abc"));
        assert!(rm.is_match(1, b"abbc"));
    }

    #[test]
    fn question_match() {
        let mut rm = RegMatch::new();
        rm.compile(1, b"ab?c").unwrap();
        assert!(rm.is_match(1, b"ac"));
        assert!(rm.is_match(1, b"abc"));
        assert!(!rm.is_match(1, b"abbc"));
    }

    #[test]
    fn char_class() {
        let mut rm = RegMatch::new();
        rm.compile(1, b"[abc]x").unwrap();
        assert!(rm.is_match(1, b"ax"));
        assert!(rm.is_match(1, b"bx"));
        assert!(!rm.is_match(1, b"dx"));
    }

    #[test]
    fn neg_class() {
        let mut rm = RegMatch::new();
        rm.compile(1, b"[^abc]x").unwrap();
        assert!(!rm.is_match(1, b"ax"));
        assert!(rm.is_match(1, b"dx"));
    }

    #[test]
    fn no_pattern() {
        let mut rm = RegMatch::new();
        assert!(!rm.is_match(1, b"hello"));
    }

    #[test]
    fn stats() {
        let mut rm = RegMatch::new();
        rm.compile(1, b"abc").unwrap();
        rm.is_match(1, b"abc");
        rm.is_match(1, b"xxx");
        assert_eq!(rm.total_compiles(), 1);
        assert_eq!(rm.total_hits(), 1);
        assert_eq!(rm.total_misses(), 1);
    }

    #[test]
    fn error_display() { assert!(Rm2Error::InvalidPattern.to_string().contains("invalid")); }
}
