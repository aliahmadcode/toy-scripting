#[derive(PartialEq, Eq, Debug)]
pub enum TokenType {
    Var,
    If,
    Else,
    While,
    And,
    Class,
    False,
    True,
    Fn,
    Nil,
    Super,
    Return,
    Or,
    This,
    For,

    Ident,
    String,
    Number,

    Plus,
    Minus,
    Star,
    Slash,
    Mod,

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

#[derive(Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenType,
    pub lexeme: String,
}

pub struct Lexer<'a> {
    src: std::iter::Peekable<std::str::Chars<'a>>,
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
            "class" => TokenType::Class,
            "false" => TokenType::False,
            "true" => TokenType::True,
            "fn" => TokenType::Fn,
            "nil" => TokenType::Nil,
            "super" => TokenType::Super,
            "return" => TokenType::Return,
            "or" => TokenType::Or,
            "this" => TokenType::This,
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
                '(' => self.single(TokenType::LeftParen, '('),
                ')' => self.single(TokenType::RightParen, ')'),
                '{' => self.single(TokenType::LeftBrace, '{'),
                '}' => self.single(TokenType::RightBrace, '}'),
                ';' => self.single(TokenType::Semicolon, ';'),
                ',' => self.single(TokenType::Comma, ','),
                '+' => self.single(TokenType::Plus, '+'),
                '-' => self.single(TokenType::Minus, '-'),
                '*' => self.single(TokenType::Star, '*'),
                '%' => self.single(TokenType::Mod, '%'),
                '.' => self.single(TokenType::Dot, '.'),

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
    fn scanning_string() {
        let code = "\"\"
            \"string\"";

        let mut lexer = Lexer::new(code);

        assert_eq!(Some(Token { kind: TokenType::String, lexeme: "".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::String, lexeme: "string".to_string() }), lexer.next());
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
        let code = "and class else false for fn if nil or return super this true var while";
        let mut lexer = Lexer::new(code);

        assert_eq!(Some(Token { kind: TokenType::And, lexeme: "and".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Class, lexeme: "class".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Else, lexeme: "else".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::False, lexeme: "false".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::For, lexeme: "for".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Fn, lexeme: "fn".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::If, lexeme: "if".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Nil, lexeme: "nil".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Or, lexeme: "or".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Return, lexeme: "return".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Super, lexeme: "super".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::This, lexeme: "this".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::True, lexeme: "true".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::Var, lexeme: "var".to_string() }), lexer.next());
        assert_eq!(Some(Token { kind: TokenType::While, lexeme: "while".to_string() }), lexer.next());
    }

    #[test]
    fn scanning_eq() {
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
}
