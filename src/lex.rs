use std::fmt;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TokenType {
    Var,
    If,
    Else,
    While,
    And,
    False,
    True,
    Fn,
    Nil,
    Return,
    Or,
    For,

    Ident,
    String,
    Number,

    Plus,
    Minus,
    Star,
    Slash,

    Ternary,
    Colon,

    Equal,
    Greater,
    Less,
    Bang,

    BangEqual,
    EqualEqual,
    LessEqual,
    GreaterEqual,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Comma,
    Dot,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenType,
    pub lexeme: String,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    pub src: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { src: input.chars().peekable() }
    }

    fn multi_op(&mut self, first: char) -> Token {
        let (single, double) = match first {
            '=' => (TokenType::Equal, TokenType::EqualEqual),
            '!' => (TokenType::Bang, TokenType::BangEqual),
            '>' => (TokenType::Greater, TokenType::GreaterEqual),
            '<' => (TokenType::Less, TokenType::LessEqual),
            _ => unreachable!(),
        };

        if self.src.next_if_eq(&'=').is_some() {
            Token { kind: double, lexeme: format!("{}=", first) }
        } else {
            Token { kind: single, lexeme: first.to_string() }
        }
    }

    fn number(&mut self, first: char) -> Token {
        let mut s = first.to_string();
        while let Some(c) = self.src.peek() {
            if c.is_ascii_digit() || *c == '.' {
                s.push(*c);
                self.src.next();
            } else {
                break;
            }
        }

        Token { kind: TokenType::Number, lexeme: s }
    }

    fn string(&mut self) -> Token {
        let mut s = String::new();

        while let Some(c) = self.src.peek() {
            if *c == '"' {
                break;
            }

            s.push(*c);
            self.src.next();
        }

        self.src.next(); // closing "

        Token { kind: TokenType::String, lexeme: s }
    }

    fn identifier(&mut self, first: char) -> Token {
        let mut s = first.to_string();

        while let Some(c) = self.src.peek() {
            if c.is_alphanumeric() || *c == '_' {
                s.push(*c);
                self.src.next();
            } else {
                break;
            }
        }

        let kind = match s.as_str() {
            "var" => TokenType::Var,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "and" => TokenType::And,
            "false" => TokenType::False,
            "true" => TokenType::True,
            "fn" => TokenType::Fn,
            "nil" => TokenType::Nil,
            "return" => TokenType::Return,
            "or" => TokenType::Or,
            _ => TokenType::Ident,
        };

        Token { kind, lexeme: s }
    }

    fn single(&self, kind: TokenType, c: char) -> Option<Token> {
        Some(Token { kind, lexeme: c.into() })
    }

    fn ignore_if_comments(&mut self, first: char) -> Option<Token> {
        if self.src.next_if_eq(&first).is_some() {
            while !self.src.next_if_eq(&'\n').is_some() {
                self.src.next();
            }

            return None;
        }

        self.single(TokenType::Slash, first)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(c) = self.src.next() {
            return match c {
                '(' => self.single(TokenType::LeftParen, c),
                ')' => self.single(TokenType::RightParen, c),
                '{' => self.single(TokenType::LeftBrace, c),
                '}' => self.single(TokenType::RightBrace, c),
                ';' => self.single(TokenType::Semicolon, c),
                ',' => self.single(TokenType::Comma, c),
                '+' => self.single(TokenType::Plus, c),
                '-' => self.single(TokenType::Minus, c),
                '*' => self.single(TokenType::Star, c),
                '.' => self.single(TokenType::Dot, c),
                '?' => self.single(TokenType::Ternary, c),
                ':' => self.single(TokenType::Colon, c),

                '/' => match self.ignore_if_comments(c) {
                    Some(t) => Some(t),
                    None => continue,
                },
                '=' | '!' | '<' | '>' => Some(self.multi_op(c)),

                '\"' => Some(self.string()),
                '0'..='9' => Some(self.number(c)),
                'a'..='z' | 'A'..='Z' | '_' => Some(self.identifier(c)),
                ' ' | '\n' | '\t' => continue,
                _ => continue,
            };
        }

        // our EOF
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanning_number() {
        let code = "
        123 // only this is allowed
        123.456 // allowed
        0.456 // allowed
        123.0 // allowed";

        let mut lexer = Lexer::new(code);

        assert_eq!(Some(Token { kind: TokenType::Number, lexeme: "123".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Number, lexeme: "123.456".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Number, lexeme: "0.456".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Number, lexeme: "123.0".to_string() }), lexer.next());
    }

    #[test]
    fn scanning_strings() {
        let code = "\"\"
            \"string\"

            var a = \"hello world\"; // a is \"hello world\"
            ";

        let mut lexer = Lexer::new(code);

        assert_eq!(Some(Token { kind: TokenType::String, lexeme: "".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::String, lexeme: "string".to_string() }), lexer.next());

        assert_eq!(Some(Token { kind: TokenType::Var, lexeme: "var".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "a".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Equal, lexeme: "=".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::String, lexeme: "hello world".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Semicolon, lexeme: ";".to_string() }), lexer.next());
    }

    #[test]
    fn scanning_whitespaces() {
        let code = "
space    tabs				newlines // removed and ignored

// hey this will be ignored
// it is also removed

a / b;

        
end
";
        let mut lexer = Lexer::new(code);
        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "space".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "tabs".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "newlines".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "a".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Slash, lexeme: "/".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "b".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Semicolon, lexeme: ";".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "end".to_string() }), lexer.next());
    }

    #[test]
    fn scanning_idents() {
        let code = "andy formless fo _ _123 _abc ab123
abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_";

        let mut lexer = Lexer::new(code);

        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "andy".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "formless".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "fo".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "_".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "_123".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "_abc".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Ident, lexeme: "ab123".to_string() }), lexer.next());
        assert_eq!(
            Some(Token {
                kind: TokenType::Ident,
                lexeme: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_".to_string()
            }),
            lexer.next()
        );
    }

    #[test]
    fn scanning_keywords() {
        let code = "and else false for fn if nil or return true var while";
        let mut lexer = Lexer::new(code);

        assert_eq!(Some(Token { kind: TokenType::And, lexeme: "and".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Else, lexeme: "else".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::False, lexeme: "false".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::For, lexeme: "for".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Fn, lexeme: "fn".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::If, lexeme: "if".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Nil, lexeme: "nil".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Or, lexeme: "or".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Return, lexeme: "return".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::True, lexeme: "true".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Var, lexeme: "var".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::While, lexeme: "while".to_string() }), lexer.next());
    }

    #[test]
    fn scanning_multi_operator() {
        let code = "(){};,+-*!===<=>=!=<>/.";
        let mut lexer = Lexer::new(code);

        assert_eq!(Some(Token { kind: TokenType::LeftParen, lexeme: "(".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::RightParen, lexeme: ")".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::LeftBrace, lexeme: "{".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::RightBrace, lexeme: "}".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Semicolon, lexeme: ";".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Comma, lexeme: ",".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Plus, lexeme: "+".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Minus, lexeme: "-".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Star, lexeme: "*".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::BangEqual, lexeme: "!=".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::EqualEqual, lexeme: "==".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::LessEqual, lexeme: "<=".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::GreaterEqual, lexeme: ">=".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::BangEqual, lexeme: "!=".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Less, lexeme: "<".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Greater, lexeme: ">".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Slash, lexeme: "/".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Dot, lexeme: ".".to_string() }), lexer.next());
    }

    #[test]
    fn scanning_expressions() {
        let code = "1.0 + 2.0 * 3.0";
        let mut lexer = Lexer::new(code);

        assert_eq!(Some(Token { kind: TokenType::Number, lexeme: "1.0".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Plus, lexeme: "+".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Number, lexeme: "2.0".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Star, lexeme: "*".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Number, lexeme: "3.0".to_string() }), lexer.next());
    }
}
