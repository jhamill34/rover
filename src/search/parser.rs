//!

use core::str::FromStr;

///
#[derive(Debug, Clone, PartialEq, Eq)]
enum PatternTokens {
    ///
    Quote,
    ///
    Caret,
    ///
    Dollar,
    ///
    Space,
    ///
    Pipe,
    ///
    Bang,
    ///
    LParen, 
    ///
    RParen,
    ///
    Star,
    ///
    Pattern(String),
}

///
struct Walker<'inner, T> {
    ///
    buffer: &'inner [T],

    ///
    current: usize,
}

impl <'inner, T: PartialEq> Walker<'inner, T> {
    ///
    fn new(buffer: &'inner [T]) -> Self {
        Self {
            buffer,
            current: 0,
        }
    }

    ///
    fn peek(&self) -> Option<&T> {
        self.buffer.get(self.current)
    }

    ///
    fn match_tokens(&mut self, tokens: &[T]) -> bool {
        let mut current = self.current;

        for token in tokens {
            if let Some(next) = self.buffer.get(current) {
                if next != token {
                    return false;
                }
            } else {
                return false;
            }

            current = current.saturating_add(1);
        }
        self.current = self.current.saturating_add(tokens.len());

        true
    }

    ///
    fn advance(&mut self) {
        self.current = self.current.saturating_add(1);
    }

    ///
    fn buffer_size(&self) -> usize {
        self.buffer.len()
    }
}

///
fn lexer<T: AsRef<[u8]>>(input: T) -> anyhow::Result<Vec<PatternTokens>> {
    let mut walker = Walker::new(input.as_ref());
    let mut tokens = Vec::with_capacity(walker.buffer_size());

    while let Some(current) = walker.peek().copied() {
        match current {
            b' ' => {
                walker.advance();

                while let Some(b' ') = walker.peek().copied() {
                    walker.advance();
                }

                tokens.push(PatternTokens::Space);
            },
            b'|' => {
                walker.advance();
                tokens.push(PatternTokens::Pipe);
            },
            b'!' => {
                walker.advance();
                tokens.push(PatternTokens::Bang);
            },
            b'*' => {
                walker.advance();
                tokens.push(PatternTokens::Star);
            },
            b'^' => {
                walker.advance();
                tokens.push(PatternTokens::Caret);
            },
            b'\'' => {
                walker.advance();
                tokens.push(PatternTokens::Quote);
            },
            b'$' => {
                walker.advance();
                tokens.push(PatternTokens::Dollar);
            },
            b'(' => {
                walker.advance();
                tokens.push(PatternTokens::LParen);
            },
            b')' => {
                walker.advance();
                tokens.push(PatternTokens::RParen);
            },
            _ => {
                let mut buffer = Vec::new();
                while let Some(current) = walker.peek().copied() {
                    if let b' ' | b'$' = current {
                        break;
                    } 

                    walker.advance();
                    buffer.push(current);
                }
                let pattern = String::from_utf8(buffer)?;
                tokens.push(PatternTokens::Pattern(pattern));
            }
        }
    }

    Ok(tokens)
}

///
fn parse(input: &[PatternTokens]) -> Pattern {
    let mut walker = Walker::new(input);
    walker.match_tokens(&[PatternTokens::Space]);
    parse_and_group(&mut walker)
}

///
fn parse_and_group(walker: &mut Walker<PatternTokens>) -> Pattern {
    let mut expr = parse_or_group(walker);

    while let Some(&PatternTokens::Space) = walker.peek() {
        walker.advance();

        if let Some(&PatternTokens::RParen) | None = walker.peek() {
            break;
        }

        let right = parse_or_group(walker);
        expr = Pattern::Group{ op: Operation::And, left: Box::new(expr), right: Box::new(right) };
    }

    expr
}

///
fn parse_or_group(walker: &mut Walker<PatternTokens>) -> Pattern {
    let mut expr = parse_negated_pattern(walker);

    while walker.match_tokens(&[PatternTokens::Space, PatternTokens::Pipe, PatternTokens::Space]) {
        let right = parse_negated_pattern(walker);
        expr = Pattern::Group{ op: Operation::Or, left: Box::new(expr), right: Box::new(right) };
    }

    expr
}

///
fn parse_negated_pattern(walker: &mut Walker<PatternTokens>) -> Pattern {
    if let Some(&PatternTokens::Bang) = walker.peek() {
        walker.advance();
        let pattern = parse_negated_pattern(walker);
        Pattern::Negated(Box::new(pattern))
    } else {
        parse_deref_pattern(walker)
    }
} 

///
fn parse_deref_pattern(walker: &mut Walker<PatternTokens>) -> Pattern {
    if let Some(&PatternTokens::Star) = walker.peek() {
        walker.advance();
        let pattern = parse_decorated_pattern(walker);
        Pattern::Deref(Box::new(pattern))
    } else {
        parse_decorated_pattern(walker)
    }
}

///
fn parse_decorated_pattern(walker: &mut Walker<PatternTokens>) -> Pattern {
    let expr = match walker.peek() {
        Some(&PatternTokens::Caret) => {
            walker.advance();
            let pattern = parse_pattern(walker);
            Pattern::Prefix(pattern)
        }
        Some(&PatternTokens::Quote) => {
            walker.advance();
            let pattern = parse_pattern(walker);
            Pattern::Exact(pattern)
        }
        Some(&PatternTokens::LParen) => {
            walker.advance();
            
            // optional spaces 
            walker.match_tokens(&[PatternTokens::Space]);

            let pattern = parse_and_group(walker);
                
            walker.match_tokens(&[PatternTokens::RParen]);
            pattern
        }
        Some(_) => {
            let pattern = parse_pattern(walker);
            if let Some(&PatternTokens::Dollar) = walker.peek() {
                walker.advance();
                Pattern::Suffix(pattern)
            } else {
                Pattern::Fuzzy(pattern)
            }
        }
        None => {
            Pattern::Fuzzy(String::new())
        }
    };

    expr
}

///
fn parse_pattern(walker: &mut Walker<PatternTokens>) -> String {
    if let Some(&PatternTokens::Pattern(ref pattern)) = walker.peek() {
        let pattern = pattern.clone();
        walker.advance();
        pattern
    } else {
        String::new()
    }
}

///
#[derive(Debug, PartialEq)]
pub enum Operation {
    ///
    And,

    ///
    Or,
}

///
#[derive(Debug, PartialEq)]
pub enum Pattern {
    ///
    Group{ 
        ///
        op: Operation, 

        ///
        left: Box<Pattern>,

        ///
        right: Box<Pattern>,
    },

    ///
    Negated(Box<Pattern>),

    ///
    Exact(String),

    ///
    Prefix(String),

    ///
    Suffix(String),

    /// 
    Fuzzy(String),

    ///
    Deref(Box<Pattern>),
}

impl FromStr for Pattern {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = lexer(s)?;
        Ok(parse(&tokens))
    }
}

#[cfg(test)]
mod test {
    use anyhow::bail;

    use super::*;

    #[test]
    fn test_lexing() -> anyhow::Result<()> {
        let tokens = lexer("'Hello   ^world | josh$ other")?;

        let expected = vec![
            PatternTokens::Quote,
            PatternTokens::Pattern("Hello".to_owned()),
            PatternTokens::Space,
            PatternTokens::Caret,
            PatternTokens::Pattern("world".to_owned()),
            PatternTokens::Space,
            PatternTokens::Pipe,
            PatternTokens::Space,
            PatternTokens::Pattern("josh".to_owned()),
            PatternTokens::Dollar,
            PatternTokens::Space,
            PatternTokens::Pattern("other".to_owned()),
        ];

        if tokens != expected {
            bail!("Expected {:?}, got {:?}", expected, tokens);
        }

        Ok(())
    }

    #[test]
    fn test_fuzzy_parsing() -> anyhow::Result<()> {
        let pattern = Pattern::from_str("Hello")?;

        let expected = Pattern::Fuzzy("Hello".to_owned());

        if pattern != expected {
            bail!("Expected {:?}, got {:?}", expected, pattern);
        }

        Ok(())
    }
    
    #[test]
    fn test_exact_parsing() -> anyhow::Result<()> {
        let pattern = Pattern::from_str("'Hello")?;

        let expected = Pattern::Exact("Hello".to_owned());

        if pattern != expected {
            bail!("Expected {:?}, got {:?}", expected, pattern);
        }

        Ok(())
    }
    
    #[test]
    fn test_prefix_parsing() -> anyhow::Result<()> {
        let pattern = Pattern::from_str("^Hello")?;

        let expected = Pattern::Prefix("Hello".to_owned());

        if pattern != expected {
            bail!("Expected {:?}, got {:?}", expected, pattern);
        }

        Ok(())
    }
    
    #[test]
    fn test_suffix_parsing() -> anyhow::Result<()> {
        let pattern = Pattern::from_str("Hello$")?;

        let expected = Pattern::Suffix("Hello".to_owned());

        if pattern != expected {
            bail!("Expected {:?}, got {:?}", expected, pattern);
        }

        Ok(())
    }
    
    #[test]
    fn test_or_parsing() -> anyhow::Result<()> {
        let pattern = Pattern::from_str("Hello | ^world")?;

        let expected = Pattern::Group { 
            op: Operation::Or,
            left: Box::new(Pattern::Fuzzy("Hello".to_owned())),
            right: Box::new(Pattern::Prefix("world".to_owned())),
        };

        if pattern != expected {
            bail!("Expected {:?}, got {:?}", expected, pattern);
        }

        Ok(())
    }

    #[test]
    fn test_and_parsing() -> anyhow::Result<()> {
        let pattern = "Hello ^world".parse::<Pattern>()?;

        let expected = Pattern::Group { 
            op: Operation::And,
            left: Box::new(Pattern::Fuzzy("Hello".to_owned())),
            right: Box::new(Pattern::Prefix("world".to_owned())),
        };

        if pattern != expected {
            bail!("Expected {:?}, got {:?}", expected, pattern);
        }

        Ok(())
    }
    
    #[test]
    fn test_order_of_operations() -> anyhow::Result<()> {
        let pattern = "Hello | ^world 'foo".parse::<Pattern>()?;

        let expected = Pattern::Group { 
            op: Operation::And,
            left: Box::new(Pattern::Group {
                op: Operation::Or,
                left: Box::new(Pattern::Fuzzy("Hello".to_owned())),
                right: Box::new(Pattern::Prefix("world".to_owned())),
            }),
            right: Box::new(Pattern::Exact("foo".to_owned())),
        };

        if pattern != expected {
            bail!("Expected {:?}, got {:?}", expected, pattern);
        }

        Ok(())
    }
   
    #[test]
    fn test_order_of_operations_with_group() -> anyhow::Result<()> {
        let pattern = "( Hello ^world ) | 'foo".parse::<Pattern>()?;

        let expected = Pattern::Group { 
            op: Operation::Or,
            left: Box::new(Pattern::Group {
                op: Operation::And,
                left: Box::new(Pattern::Fuzzy("Hello".to_owned())),
                right: Box::new(Pattern::Prefix("world".to_owned())),
            }),
            right: Box::new(Pattern::Exact("foo".to_owned())),
        };

        if pattern != expected {
            bail!("Expected {:?}, got {:?}", expected, pattern);
        }

        Ok(())
    }

    #[test]
    fn test_negated_pattern() -> anyhow::Result<()> {
        let pattern = "!'Hello".parse::<Pattern>()?;

        let expected = Pattern::Negated(
            Box::new(Pattern::Exact("Hello".to_owned()))
        );

        if pattern != expected {
            bail!("Expected {:?}, got {:?}", expected, pattern);
        }

        Ok(())
    }
    
    #[test]
    fn test_trailing_space() -> anyhow::Result<()> {
        let pattern = "!'Hello  ".parse::<Pattern>()?;

        let expected = Pattern::Negated(
            Box::new(Pattern::Exact("Hello".to_owned()))
        );

        if pattern != expected {
            bail!("Expected {:?}, got {:?}", expected, pattern);
        }

        Ok(())
    }
    
    #[test]
    fn test_leading_space() -> anyhow::Result<()> {
        let pattern = "   !'Hello".parse::<Pattern>()?;

        let expected = Pattern::Negated(
            Box::new(Pattern::Exact("Hello".to_owned()))
        );

        if pattern != expected {
            bail!("Expected {:?}, got {:?}", expected, pattern);
        }

        Ok(())
    }
    
    #[test]
    fn test_unclosed_group() -> anyhow::Result<()> {
        let pattern = "(!'Hello".parse::<Pattern>()?;

        let expected = Pattern::Negated(
            Box::new(Pattern::Exact("Hello".to_owned()))
        );

        if pattern != expected {
            bail!("Expected {:?}, got {:?}", expected, pattern);
        }

        Ok(())
    }
    
    #[test]
    fn test_deref_pattern() -> anyhow::Result<()> {
        let pattern = "!*Hello".parse::<Pattern>()?;

        let expected = Pattern::Negated(
            Box::new(Pattern::Deref(
                Box::new(Pattern::Fuzzy("Hello".to_owned()))
            )));

        if pattern != expected {
            bail!("Expected {:?}, got {:?}", expected, pattern);
        }

        Ok(())
    }
}
