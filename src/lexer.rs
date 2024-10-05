use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum Token {
    LParen,
    RParen,
    Symbol(String),
    Number(String),
    String(String),
    Quote,
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub tok: Token,
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl SpannedToken {
    pub fn new(
        tok: Token,
        start: (usize, usize),
        end: (usize, usize),
    ) -> Self {
        Self { tok, start, end }
    }

    pub fn match_punct(
        c: char,
        pos: (usize, usize),
    ) -> Option<Self> {
        match c {
            '(' => {
                Some(SpannedToken::new(Token::LParen, pos, pos))
            }
            ')' => {
                Some(SpannedToken::new(Token::RParen, pos, pos))
            }
            '\'' => {
                Some(SpannedToken::new(Token::Quote, pos, pos))
            }
            _ => None,
        }
    }
}

fn lex_punct(
    stream: &mut Peekable<
        impl Iterator<Item = (usize, usize, char)>,
    >,
) -> Option<SpannedToken> {
    let tok = stream.peek().and_then(|(line, col, c)| {
        SpannedToken::match_punct(*c, (*line, *col))
    });

    stream.next_if(|_| tok.is_some());
    tok
}

fn lex_number(
    stream: &mut Peekable<
        impl Iterator<Item = (usize, usize, char)>,
    >,
) -> Option<SpannedToken> {
    let mut number = String::new();
    let start = *stream.peek()?;

    while let Some(&(_, _, c)) = stream.peek() {
        if c.is_ascii_digit() || c == '.' {
            number.push(c);
            stream.next();
        } else {
            break;
        }
    }

    if number.is_empty() {
        None
    } else {
        let end = stream.peek().unwrap_or(&start);
        Some(SpannedToken::new(
            Token::Number(number),
            (start.0, start.1),
            // FIXME: Gotta rollback properly
            (end.0, end.1 - 1),
        ))
    }
}

fn lex_symbol(
    stream: &mut Peekable<
        impl Iterator<Item = (usize, usize, char)>,
    >,
) -> Option<SpannedToken> {
    let mut symbol = String::new();
    let start = *stream.peek()?;

    while let Some(&(_, _, c)) = stream.peek() {
        if c.is_alphanumeric()
            || "!$%&*+-./:<=>?@^_~".contains(c)
        {
            symbol.push(c);
            stream.next();
        } else {
            break;
        }
    }

    if symbol.is_empty() {
        None
    } else {
        let end = stream.peek().unwrap_or(&start);
        Some(SpannedToken::new(
            Token::Symbol(symbol),
            (start.0, start.1),
            // FIXME: Gotta rollback properly
            (end.0, end.1 - 1),
        ))
    }
}

fn lex_string(
    stream: &mut Peekable<
        impl Iterator<Item = (usize, usize, char)>,
    >,
) -> Option<SpannedToken> {
    if stream.peek().map_or(false, |&(_, _, c)| c == '"') {
        let start = *stream.peek()?;
        stream.next(); // Consume opening quote
        let mut string = String::new();

        while let Some(&(_, _, c)) = stream.peek() {
            match c {
                '"' => {
                    let end =
                        stream.next().expect("BUG: Impossible"); // Consume closing quote
                    return Some(SpannedToken::new(
                        Token::String(string),
                        (start.0, start.1),
                        (end.0, end.1),
                    ));
                }
                '\\' => {
                    string.push(c);
                    stream.next();
                    if let Some(&(_, _, next)) = stream.peek() {
                        string.push(next);
                        stream.next();
                    }
                }
                _ => {
                    string.push(c);
                    stream.next();
                }
            }
        }
    }
    None
}

pub fn lex(
    input: impl Iterator<Item = char>,
) -> impl Iterator<Item = SpannedToken> {
    let mut numbered = input
        .scan((0, 0), |(line, col), c| {
            let current = (*line, *col);
            if c == '\n' {
                *line += 1;
                *col = 0;
            } else {
                *col += 1;
            }

            Some((current.0, current.1, c))
        })
        .inspect(|x| println!("{:?}", x))
        .peekable();

    let mut tokens = vec![];

    #[allow(clippy::option_map_unit_fn)]
    while numbered.peek().is_some() {
        lex_punct(&mut numbered).map(|t| tokens.push(t));
        lex_number(&mut numbered).map(|t| tokens.push(t));
        lex_symbol(&mut numbered).map(|t| tokens.push(t));
        lex_string(&mut numbered).map(|t| tokens.push(t));

        numbered.next_if(|(_, _, c)| c.is_whitespace());
    }

    tokens.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_tokens(
        actual: Vec<SpannedToken>,
        expected: Vec<SpannedToken>,
    ) {
        println!("{:?}", &actual);
        println!("{:?}", &expected);
        assert_eq!(actual.len(), expected.len());
        for (a, e) in
            actual.into_iter().zip(expected.into_iter())
        {
            assert_eq!(a.start, e.start);
            assert_eq!(a.end, e.end);
            match (a.tok, e.tok) {
                (Token::LParen, Token::LParen) => {}
                (Token::RParen, Token::RParen) => {}
                (Token::Quote, Token::Quote) => {}
                (Token::Symbol(a_s), Token::Symbol(e_s)) => {
                    assert_eq!(a_s, e_s)
                }
                (Token::Number(a_n), Token::Number(e_n)) => {
                    assert_eq!(a_n, e_n)
                }
                (Token::String(a_s), Token::String(e_s)) => {
                    assert_eq!(a_s, e_s)
                }
                _ => panic!("Token mismatch"),
            }
        }
    }

    #[test]
    fn test_simple_expression() {
        let input = "(+ 1 2)".chars();
        let expected = vec![
            SpannedToken::new(Token::LParen, (0, 0), (0, 0)),
            SpannedToken::new(
                Token::Symbol("+".to_string()),
                (0, 1),
                (0, 1),
            ),
            SpannedToken::new(
                Token::Number("1".to_string()),
                (0, 3),
                (0, 3),
            ),
            SpannedToken::new(
                Token::Number("2".to_string()),
                (0, 5),
                (0, 5),
            ),
            SpannedToken::new(Token::RParen, (0, 6), (0, 6)),
        ];
        compare_tokens(lex(input).collect(), expected);
    }

    #[test]
    fn test_string_and_quote() {
        let input = r#"'(hello "world\n")"#.chars();
        let expected = vec![
            SpannedToken::new(Token::Quote, (0, 0), (0, 0)),
            SpannedToken::new(Token::LParen, (0, 1), (0, 1)),
            SpannedToken::new(
                Token::Symbol("hello".to_string()),
                (0, 2),
                (0, 6),
            ),
            SpannedToken::new(
                Token::String("world\\n".to_string()),
                (0, 8),
                (0, 16),
            ),
            SpannedToken::new(Token::RParen, (0, 17), (0, 17)),
        ];
        compare_tokens(lex(input).collect(), expected);
    }

    #[test]
    fn test_multiline_input() {
        let input = "(define (factorial n)\n  (if (= n 0)\n      1\n      (* n (factorial (- n 1)))))".chars();
        let expected = vec![
            SpannedToken::new(Token::LParen, (0, 0), (0, 0)),
            SpannedToken::new(
                Token::Symbol("define".to_string()),
                (0, 1),
                (0, 6),
            ),
            SpannedToken::new(Token::LParen, (0, 8), (0, 8)),
            SpannedToken::new(
                Token::Symbol("factorial".to_string()),
                (0, 9),
                (0, 17),
            ),
            SpannedToken::new(
                Token::Symbol("n".to_string()),
                (0, 19),
                (0, 19),
            ),
            SpannedToken::new(Token::RParen, (0, 20), (0, 20)),
            SpannedToken::new(Token::LParen, (1, 2), (1, 2)),
            SpannedToken::new(
                Token::Symbol("if".to_string()),
                (1, 3),
                (1, 4),
            ),
            SpannedToken::new(Token::LParen, (1, 6), (1, 6)),
            SpannedToken::new(
                Token::Symbol("=".to_string()),
                (1, 7),
                (1, 7),
            ),
            SpannedToken::new(
                Token::Symbol("n".to_string()),
                (1, 9),
                (1, 9),
            ),
            SpannedToken::new(
                Token::Number("0".to_string()),
                (1, 11),
                (1, 11),
            ),
            SpannedToken::new(Token::RParen, (1, 12), (1, 12)),
            SpannedToken::new(
                Token::Number("1".to_string()),
                (2, 6),
                (2, 6),
            ),
            SpannedToken::new(Token::LParen, (3, 6), (3, 6)),
            SpannedToken::new(
                Token::Symbol("*".to_string()),
                (3, 7),
                (3, 7),
            ),
            SpannedToken::new(
                Token::Symbol("n".to_string()),
                (3, 9),
                (3, 9),
            ),
            SpannedToken::new(Token::LParen, (3, 11), (3, 11)),
            SpannedToken::new(
                Token::Symbol("factorial".to_string()),
                (3, 12),
                (3, 20),
            ),
            SpannedToken::new(Token::LParen, (3, 22), (3, 22)),
            SpannedToken::new(
                Token::Symbol("-".to_string()),
                (3, 23),
                (3, 23),
            ),
            SpannedToken::new(
                Token::Symbol("n".to_string()),
                (3, 25),
                (3, 25),
            ),
            SpannedToken::new(
                Token::Number("1".to_string()),
                (3, 27),
                (3, 27),
            ),
            SpannedToken::new(Token::RParen, (3, 28), (3, 28)),
            SpannedToken::new(Token::RParen, (3, 29), (3, 29)),
            SpannedToken::new(Token::RParen, (3, 30), (3, 30)),
            SpannedToken::new(Token::RParen, (3, 31), (3, 31)),
            SpannedToken::new(Token::RParen, (3, 32), (3, 32)),
        ];
        compare_tokens(lex(input).collect(), expected);
    }

    #[test]
    fn test_edge_cases() {
        let input = r#"(symbol1Symbol2 3.14 "string with \"escape\"" '())"#.chars();
        let expected = vec![
            SpannedToken::new(Token::LParen, (0, 0), (0, 0)),
            SpannedToken::new(
                Token::Symbol("symbol1Symbol2".to_string()),
                (0, 1),
                (0, 14),
            ),
            SpannedToken::new(
                Token::Number("3.14".to_string()),
                (0, 16),
                (0, 19),
            ),
            SpannedToken::new(
                Token::String(
                    "string with \\\"escape\\\"".to_string(),
                ),
                (0, 21),
                (0, 44),
            ),
            SpannedToken::new(Token::Quote, (0, 46), (0, 46)),
            SpannedToken::new(Token::LParen, (0, 47), (0, 47)),
            SpannedToken::new(Token::RParen, (0, 48), (0, 48)),
            SpannedToken::new(Token::RParen, (0, 49), (0, 49)),
        ];
        compare_tokens(lex(input).collect(), expected);
    }
}
