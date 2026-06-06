use std::collections::HashSet;

/// Parsed SPDX license expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpdxExpr {
    /// A single license ID, optionally with a "+" for "or later".
    License { id: String, plus: bool },
    /// A WITH exception: License WITH Exception
    WithException { license: Box<SpdxExpr>, exception: String },
    /// AND conjunction: both licenses apply.
    And(Box<SpdxExpr>, Box<SpdxExpr>),
    /// OR disjunction: either license applies.
    Or(Box<SpdxExpr>, Box<SpdxExpr>),
}

impl SpdxExpr {
    /// Collect all unique license IDs in this expression.
    pub fn all_license_ids(&self) -> Vec<String> {
        let mut ids = HashSet::new();
        self.collect_ids(&mut ids);
        ids.into_iter().collect()
    }

    fn collect_ids(&self, ids: &mut HashSet<String>) {
        match self {
            SpdxExpr::License { id, .. } => {
                ids.insert(id.clone());
            }
            SpdxExpr::WithException { license, .. } => {
                license.collect_ids(ids);
            }
            SpdxExpr::And(a, b) | SpdxExpr::Or(a, b) => {
                a.collect_ids(ids);
                b.collect_ids(ids);
            }
        }
    }

    /// Check if this expression contains any OR operators.
    pub fn has_or(&self) -> bool {
        match self {
            SpdxExpr::Or(_, _) => true,
            SpdxExpr::And(a, b) => a.has_or() || b.has_or(),
            SpdxExpr::WithException { license, .. } => license.has_or(),
            SpdxExpr::License { .. } => false,
        }
    }
}

/// Parser for SPDX license expressions.
pub struct SpdxParser;

impl SpdxParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse an SPDX license expression string.
    pub fn parse(&self, input: &str) -> SpdxExpr {
        let tokens = self.tokenize(input.trim());
        self.parse_tokens(&tokens).unwrap_or_else(|| SpdxExpr::License {
            id: input.trim().to_string(),
            plus: false,
        })
    }

    /// Tokenize an SPDX expression.
    fn tokenize(&self, input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '(' => {
                    if !current.is_empty() {
                        tokens.push(Token::Id(self.normalize_id(&current)));
                        current.clear();
                    }
                    tokens.push(Token::LParen);
                }
                ')' => {
                    if !current.is_empty() {
                        tokens.push(Token::Id(self.normalize_id(&current)));
                        current.clear();
                    }
                    tokens.push(Token::RParen);
                }
                '+' => {
                    if !current.is_empty() {
                        tokens.push(Token::Id(self.normalize_id(&current)));
                        current.clear();
                    }
                    tokens.push(Token::Plus);
                }
                ' ' | '\t' => {
                    if !current.is_empty() {
                        let upper = current.to_uppercase();
                        if upper == "AND" {
                            tokens.push(Token::And);
                        } else if upper == "OR" {
                            tokens.push(Token::Or);
                        } else if upper == "WITH" {
                            tokens.push(Token::With);
                        } else {
                            tokens.push(Token::Id(self.normalize_id(&current)));
                        }
                        current.clear();
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.is_empty() {
            let upper = current.to_uppercase();
            if upper == "AND" {
                tokens.push(Token::And);
            } else if upper == "OR" {
                tokens.push(Token::Or);
            } else if upper == "WITH" {
                tokens.push(Token::With);
            } else {
                tokens.push(Token::Id(self.normalize_id(&current)));
            }
        }

        tokens
    }

    /// Normalize a license ID (e.g., "mit" -> "MIT").
    fn normalize_id(&self, id: &str) -> String {
        // Handle common variations
        let id = id.trim();
        // Known SPDX IDs - return canonical form
        match id.to_uppercase().as_str() {
            "MIT" => "MIT".into(),
            "APACHE-2.0" | "APACHE2" | "APACHE-2" => "Apache-2.0".into(),
            "GPL-2.0" => "GPL-2.0".into(),
            "GPL-3.0" => "GPL-3.0".into(),
            "AGPL-3.0" => "AGPL-3.0".into(),
            "LGPL-2.1" => "LGPL-2.1".into(),
            "LGPL-3.0" => "LGPL-3.0".into(),
            "BSD-2-CLAUSE" => "BSD-2-Clause".into(),
            "BSD-3-CLAUSE" => "BSD-3-Clause".into(),
            "MPL-2.0" => "MPL-2.0".into(),
            "ISC" => "ISC".into(),
            "UNLICENSE" => "Unlicense".into(),
            "CC0-1.0" => "CC0-1.0".into(),
            "0BSD" => "0BSD".into(),
            "BSL-1.0" => "BSL-1.0".into(),
            "ZLIB" => "Zlib".into(),
            "BLUEOAK-1.0.0" => "BlueOak-1.0.0".into(),
            "MIT-0" => "MIT-0".into(),
            _ => id.into(),
        }
    }

    /// Recursive descent parser for tokens.
    fn parse_tokens(&self, tokens: &[Token]) -> Option<SpdxExpr> {
        if tokens.is_empty() {
            return None;
        }

        let (expr, remaining) = self.parse_or(tokens)?;
        if remaining.is_empty() {
            Some(expr)
        } else {
            // Try to continue parsing
            Some(expr)
        }
    }

    fn parse_or<'a>(&self, tokens: &'a [Token]) -> Option<(SpdxExpr, &'a [Token])> {
        let (mut left, mut remaining) = self.parse_and(tokens)?;

        while !remaining.is_empty() {
            if matches!(remaining.first(), Some(Token::Or)) {
                let (right, rest) = self.parse_and(&remaining[1..])?;
                left = SpdxExpr::Or(Box::new(left), Box::new(right));
                remaining = rest;
            } else {
                break;
            }
        }

        Some((left, remaining))
    }

    fn parse_and<'a>(&self, tokens: &'a [Token]) -> Option<(SpdxExpr, &'a [Token])> {
        let (mut left, mut remaining) = self.parse_with(tokens)?;

        while !remaining.is_empty() {
            if matches!(remaining.first(), Some(Token::And)) {
                let (right, rest) = self.parse_with(&remaining[1..])?;
                left = SpdxExpr::And(Box::new(left), Box::new(right));
                remaining = rest;
            } else {
                break;
            }
        }

        Some((left, remaining))
    }

    fn parse_with<'a>(&self, tokens: &'a [Token]) -> Option<(SpdxExpr, &'a [Token])> {
        let (mut left, mut remaining) = self.parse_primary(tokens)?;

        while !remaining.is_empty() {
            if matches!(remaining.first(), Some(Token::With)) {
                // Next should be the exception identifier
                if remaining.len() >= 2 {
                    if let Token::Id(exception) = &remaining[1] {
                        left = SpdxExpr::WithException {
                            license: Box::new(left),
                            exception: exception.clone(),
                        };
                        remaining = &remaining[2..];
                        continue;
                    }
                }
                break;
            } else {
                break;
            }
        }

        Some((left, remaining))
    }

    fn parse_primary<'a>(&self, tokens: &'a [Token]) -> Option<(SpdxExpr, &'a [Token])> {
        if tokens.is_empty() {
            return None;
        }

        match &tokens[0] {
            Token::LParen => {
                let (expr, remaining) = self.parse_or(&tokens[1..])?;
                // Expect closing paren
                if matches!(remaining.first(), Some(Token::RParen)) {
                    Some((expr, &remaining[1..]))
                } else {
                    Some((expr, remaining))
                }
            }
            Token::Id(id) => {
                let plus = matches!(tokens.get(1), Some(Token::Plus));
                let consume = if plus { 2 } else { 1 };
                Some((
                    SpdxExpr::License {
                        id: id.clone(),
                        plus,
                    },
                    &tokens[consume..],
                ))
            }
            _ => None,
        }
    }
}

impl Default for SpdxParser {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Id(String),
    And,
    Or,
    With,
    Plus,
    LParen,
    RParen,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_license() {
        let parser = SpdxParser::new();
        let expr = parser.parse("MIT");
        assert_eq!(expr, SpdxExpr::License { id: "MIT".into(), plus: false });
    }

    #[test]
    fn test_parse_or_expression() {
        let parser = SpdxParser::new();
        let expr = parser.parse("MIT OR Apache-2.0");
        assert!(matches!(expr, SpdxExpr::Or(_, _)));
        let ids = expr.all_license_ids();
        assert!(ids.contains(&"MIT".to_string()));
        assert!(ids.contains(&"Apache-2.0".to_string()));
    }

    #[test]
    fn test_parse_and_expression() {
        let parser = SpdxParser::new();
        let expr = parser.parse("MIT AND Apache-2.0");
        assert!(matches!(expr, SpdxExpr::And(_, _)));
    }

    #[test]
    fn test_parse_plus() {
        let parser = SpdxParser::new();
        let expr = parser.parse("GPL-3.0+");
        assert_eq!(expr, SpdxExpr::License { id: "GPL-3.0".into(), plus: true });
    }

    #[test]
    fn test_parse_with_exception() {
        let parser = SpdxParser::new();
        let expr = parser.parse("GPL-2.0 WITH Classpath-exception-2.0");
        assert!(matches!(expr, SpdxExpr::WithException { .. }));
        let ids = expr.all_license_ids();
        assert!(ids.contains(&"GPL-2.0".to_string()));
    }

    #[test]
    fn test_parse_parenthesized() {
        let parser = SpdxParser::new();
        let expr = parser.parse("(MIT OR Apache-2.0) AND BSD-3-Clause");
        assert!(matches!(expr, SpdxExpr::And(_, _)));
        assert!(expr.has_or());
    }

    #[test]
    fn test_all_license_ids_complex() {
        let parser = SpdxParser::new();
        let expr = parser.parse("MIT OR Apache-2.0 AND BSD-3-Clause");
        let ids = expr.all_license_ids();
        assert!(ids.contains(&"MIT".to_string()));
        assert!(ids.contains(&"Apache-2.0".to_string()));
        assert!(ids.contains(&"BSD-3-Clause".to_string()));
    }

    #[test]
    fn test_has_or() {
        let parser = SpdxParser::new();
        assert!(parser.parse("MIT OR Apache-2.0").has_or());
        assert!(!parser.parse("MIT").has_or());
        assert!(!parser.parse("MIT AND BSD-3-Clause").has_or());
    }

    #[test]
    fn test_normalize_apache() {
        let parser = SpdxParser::new();
        let expr = parser.parse("Apache2");
        assert_eq!(expr, SpdxExpr::License { id: "Apache-2.0".into(), plus: false });
    }

    #[test]
    fn test_parse_complex_or_chain() {
        let parser = SpdxParser::new();
        let expr = parser.parse("Unlicense OR MIT");
        let ids = expr.all_license_ids();
        assert!(ids.contains(&"Unlicense".to_string()));
        assert!(ids.contains(&"MIT".to_string()));
    }
}
