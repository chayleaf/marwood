use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenType {
    Dot,
    False,
    LeftParen,
    Number,
    RightParen,
    SingleQuote,
    Symbol,
    True,
    WhiteSpace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub span: (usize, usize),
    pub token_type: TokenType,
}

impl Token {
    pub fn new(span: (usize, usize), token_type: TokenType) -> Token {
        Token { span, token_type }
    }

    pub fn span_text<'a, 'b>(&'a self, text: &'b str) -> &'b str {
        &text[self.span.0..self.span.1]
    }
}

pub fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut cur = text.char_indices().peekable();

    while let Some(&(offset, c)) = cur.peek() {
        let token = match c {
            '(' | ')' | '\'' | '.' => {
                cur.next();
                let token_type = match c {
                    '(' => TokenType::LeftParen,
                    ')' => TokenType::RightParen,
                    '\'' => TokenType::SingleQuote,
                    '.' => TokenType::Dot,
                    _ => {
                        continue;
                    }
                };
                Some(Token::new((offset, offset + c.len_utf8()), token_type))
            }
            '#' => {
                cur.next();
                match cur.next() {
                    Some((offset, c_next)) => match c {
                        't' => Some(Token::new(
                            (offset, offset + c.len_utf8() + c_next.len_utf16()),
                            TokenType::True,
                        )),
                        'f' => Some(Token::new(
                            (offset, offset + c.len_utf8() + c_next.len_utf16()),
                            TokenType::False,
                        )),
                        _ => None,
                    },
                    None => None,
                }
            }
            _ if is_initial_identifier(c) => Some(scan_symbol(&mut cur)),
            _ if is_initial_number(c) => Some(scan_number(&mut cur)),
            _ if c.is_whitespace() => {
                cur.next();
                None
            }
            _ => {
                panic!("unknown character {}", c);
            }
        };
        if let Some(token) = token {
            tokens.push(token);
        }
    }
    tokens
}

fn scan_symbol(cur: &mut Peekable<CharIndices>) -> Token {
    let start = cur.peek().unwrap().0;
    let mut end = start;
    while let Some(&(offset, c)) = cur.peek() {
        if !is_subsequent_identifier(c) && start != end {
            break;
        }
        end = offset + c.len_utf8();
        cur.next();
    }
    Token::new((start, end), TokenType::Symbol)
}

fn scan_number(cur: &mut Peekable<CharIndices>) -> Token {
    let start = cur.peek().unwrap().0;
    let mut end = start;
    while let Some(&(offset, c)) = cur.peek() {
        if !is_subsequent_number(c) && start != end {
            break;
        }
        end = offset + c.len_utf8();
        cur.next();
    }
    Token::new((start, end), TokenType::Number)
}

fn is_initial_number(c: char) -> bool {
    c.is_digit(10) || c == '+' || c == '-'
}

fn is_subsequent_number(c: char) -> bool {
    c.is_digit(10) || c == '.'
}

fn is_initial_identifier(c: char) -> bool {
    c.is_alphabetic()
        || c == '!'
        || c == '$'
        || c == '%'
        || c == '&'
        || c == '*'
        || c == '/'
        || c == ':'
        || c == '<'
        || c == '='
        || c == '>'
        || c == '?'
        || c == '^'
        || c == '_'
        || c == '~'
}

fn is_special_subsequent(c: char) -> bool {
    c == '+' || c == '-' || c == '.' || c == '@'
}

fn is_subsequent_identifier(c: char) -> bool {
    is_initial_identifier(c) || c.is_digit(10) || is_special_subsequent(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Map tokens to a vector (text, type) pairs.
    fn expand<T: IntoIterator<Item = Token>>(
        tokens: T,
        original_text: &str,
    ) -> Vec<(&str, TokenType)> {
        tokens
            .into_iter()
            .map(|token| (token.span_text(original_text), token.token_type.clone()))
            .collect()
    }

    macro_rules! lexes {
        ($lhs:expr => $(($token_text:expr, $token_type:expr)),+) => {{
            let mut v = vec![];
            $(v.push(($token_text, $token_type));)+
            assert_eq!(expand(tokenize($lhs), $lhs), v);
        }};
        ($($lhs:expr => $rhs:expr),+) => {{
             $(
                assert_eq!(expand(tokenize($lhs), $lhs).iter().next().unwrap(), &($lhs, $rhs));
             )+
        }};
    }

    #[test]
    fn parens() {
        lexes!(
            "(" => TokenType::LeftParen,
            ")" => TokenType::RightParen
        );
    }

    #[test]
    fn symbols() {
        lexes!(
            "foo" => TokenType::Symbol
        );
    }

    #[test]
    fn numbers() {
        lexes!(
            "5" => TokenType::Number,
            "10" => TokenType::Number,
            "10.5" => TokenType::Number,
            "10..5" => TokenType::Number,
            "-42" => TokenType::Number,
            "+42" => TokenType::Number,
            "-10.5" => TokenType::Number
        )
    }

    #[test]
    fn dot() {
        lexes!(
            "(foo . bar)" =>
            ("(", TokenType::LeftParen),
            ("foo", TokenType::Symbol),
            (".", TokenType::Dot),
            ("bar", TokenType::Symbol),
            (")", TokenType::RightParen)
        );
    }

    #[test]
    fn multiple_expressions() {
        lexes!(
            "42.0 '((1024)('baz))" =>
            ("42.0", TokenType::Number),
            ("'", TokenType::SingleQuote),
            ("(", TokenType::LeftParen),
            ("(", TokenType::LeftParen),
            ("1024", TokenType::Number),
            (")", TokenType::RightParen),
            ("(", TokenType::LeftParen),
            ("'", TokenType::SingleQuote),
            ("baz", TokenType::Symbol),
            (")", TokenType::RightParen),
            (")", TokenType::RightParen)
        );
    }

    #[test]
    fn boolean_tokens() {
        lexes!(
            "#t" => TokenType::True,
            "#f" => TokenType::False
        )
    }
}
