pub mod lex;
use std::fmt;

use std::iter::Peekable;

use lex::{Lexer, Token, TokenType};

#[derive(PartialEq, Debug)]
pub enum Expr {
    Number(f64),
    Ident(String),
    Binary(TokenType, Vec<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(n) => write!(f, " {}", n),
            Expr::Ident(s) => write!(f, " {}", s),
            Expr::Binary(token_type, exprs) => {
                write!(f, "({}", token_type)?;
                for s in exprs {
                    write!(f, "{}", s)?;
                }
                write!(f, ")")
            }
        }
    }
}

pub fn expr(input: &str) -> Expr {
    let mut lexer = Lexer::new(input).peekable();
    expr_bp(&mut lexer, 0)
}

fn expr_bp(lexer: &mut Peekable<Lexer<'_>>, min_bp: u8) -> Expr {
    let Some(t) = lexer.next() else {
        panic!("unknown token");
    };

    let mut lhs = match t.kind {
        TokenType::Number => Expr::Number(t.lexeme.parse::<f64>().expect("can't parse number")),
        TokenType::Ident => Expr::Ident(t.lexeme.to_string()),
        TokenType::Plus | TokenType::Minus => {
            let op = t.kind.clone();
            let ((), r_bp) = prefix_binding_power(&op);
            let rhs = expr_bp(lexer, r_bp);
            Expr::Binary(op, vec![rhs])
        }
        TokenType::LeftParen => {
            let lhs = expr_bp(lexer, 0);
            assert_eq!(lexer.next(), Some(Token { kind: TokenType::RightParen, lexeme: ")".into() }));
            lhs
        }
        _ => unreachable!(),
    };

    loop {
        let Some(t) = lexer.peek() else {
            break;
        };

        let op = t.kind.clone();

        if let Some((l_bp, ())) = postfix_binding_power(&op) {
            if l_bp < min_bp {
                break;
            }

            lexer.next();

            lhs = if op == TokenType::LeftBrace {
                let rhs = expr_bp(lexer, 0);
                assert_eq!(lexer.next(), Some(Token { kind: TokenType::RightBrace, lexeme: "}".into() }));
                Expr::Binary(op, vec![lhs, rhs])
            } else {
                Expr::Binary(op, vec![lhs])
            };

            continue;
        }

        if let Some((l_bp, r_bp)) = infix_binding_power(&op) {
            if l_bp < min_bp {
                break;
            }

            lexer.next();

            lhs = if op == TokenType::Ternary {
                let mhs = expr_bp(lexer, 0);
                assert_eq!(lexer.next(), Some(Token { kind: TokenType::Colon, lexeme: ":".into() }));
                let rhs = expr_bp(lexer, r_bp);
                Expr::Binary(op, vec![lhs, mhs, rhs])
            } else {
                let rhs = expr_bp(lexer, r_bp);
                Expr::Binary(op, vec![lhs, rhs])
            };

            continue;
        }

        break;
    }

    lhs
}

fn prefix_binding_power(op: &TokenType) -> ((), u8) {
    match op {
        TokenType::Plus | TokenType::Minus => ((), 9),
        _ => panic!("bad op: {}", op),
    }
}

fn postfix_binding_power(op: &TokenType) -> Option<(u8, ())> {
    match op {
        TokenType::Bang | TokenType::LeftBrace => Some((11, ())),
        _ => None,
    }
}

fn infix_binding_power(op: &TokenType) -> Option<(u8, u8)> {
    match op {
        TokenType::Equal => Some((2, 1)),
        TokenType::Ternary => Some((4, 3)),
        TokenType::Plus | TokenType::Minus => Some((5, 6)),
        TokenType::Star | TokenType::Slash => Some((7, 8)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expr_bp() {
        let x = expr("1");
        assert_eq!(x.to_string(), " 1");

        let x = expr("((((0))))");
        assert_eq!(x.to_string(), " 0");

        let x = expr("x + 1");
        assert_eq!(x.to_string(), "(Plus x 1)");

        let x = expr("x{0}{1}");
        assert_eq!(x.to_string(), "(LeftBrace(LeftBrace x 0) 1)");

        let x = expr("1 + 2 * 3 - 3 / 4");
        assert_eq!(x.to_string(), "(Minus(Plus 1(Star 2 3))(Slash 3 4))");

        let x = expr("- 1 + 2");
        assert_eq!(x.to_string(), "(Plus(Minus 1) 2)");

        let x = expr("- 1! + 2");
        assert_eq!(x.to_string(), "(Plus(Minus(Bang 1)) 2)");

        let x = expr(
            "a ? 1 + 2 :
         c ? d
         : e",
        );
        assert_eq!(x.to_string(), "(Ternary a(Plus 1 2)(Ternary c d e))");

        let x = expr("a = 0 ? b : c = d");
        assert_eq!(x.to_string(), "(Equal a(Equal(Ternary 0 b c) d))");
    }
}
