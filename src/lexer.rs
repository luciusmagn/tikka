use itertools::Itertools;

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
}

#[derive(Debug, Clone)]
enum LexState {
    Normal,
    InString(bool), // bool indicates if prev char was escape
    InSymbol,
    InNumber,
}

#[derive(Debug, Clone)]
enum LexItem {
    Token(SpannedToken),
    Char(char, (usize, usize)),
    StringStart((usize, usize)),
    StringEnd,
}

pub fn lex(
    input: impl Iterator<Item = char>,
) -> impl Iterator<Item = SpannedToken> {
    let mut position = (0, 0);

    let mut buffer = vec![];
    let mut state = LexState::Normal;
    let mut tokens = vec![];

    let mut mark = position;

    for c in input {
        use LexState::*;
        eprintln!("{}{:?}", c, state);
        match state {
            Normal => match c {
                '(' => tokens.push(SpannedToken::new(
                    Token::LParen,
                    position,
                    position,
                )),
                ')' => tokens.push(SpannedToken::new(
                    Token::RParen,
                    position,
                    position,
                )),
                '\'' => tokens.push(SpannedToken::new(
                    Token::Quote,
                    position,
                    position,
                )),
                '"' => state = InString(false),
                c if c.is_whitespace() => (),
                c if c.is_digit(10) => {
                    state = InNumber;

                    mark = position;
                    buffer.clear();
                    buffer.push(c);
                }
                c => {
                    mark = position;
                    state = InSymbol;

                    buffer.clear();
                    buffer.push(c);
                }
            },
            InSymbol => match c {
                c if c.is_whitespace() => {
                    tokens.push(SpannedToken::new(
                        Token::Symbol(
                            buffer.clone().iter().collect(),
                        ),
                        mark,
                        // FIXME: this will be bad if we are at the beginning of the line
                        (position.0, position.1 - 1),
                    ));

                    state = Normal;
                }
                '(' => {
                    tokens.push(SpannedToken::new(
                        Token::Symbol(
                            buffer.clone().iter().collect(),
                        ),
                        mark,
                        // FIXME: this will be bad if we are at the beginning of the line
                        (position.0, position.1 - 1),
                    ));

                    tokens.push(SpannedToken::new(
                        Token::LParen,
                        position,
                        position,
                    ));

                    state = Normal;
                }
                ')' => {
                    tokens.push(SpannedToken::new(
                        Token::Symbol(
                            buffer.clone().iter().collect(),
                        ),
                        mark,
                        // FIXME: this will be bad if we are at the beginning of the line
                        (position.0, position.1 - 1),
                    ));

                    tokens.push(SpannedToken::new(
                        Token::RParen,
                        position,
                        position,
                    ));

                    state = Normal;
                }
                '\'' => {
                    tokens.push(SpannedToken::new(
                        Token::Symbol(
                            buffer.clone().iter().collect(),
                        ),
                        mark,
                        // FIXME: this will be bad if we are at the beginning of the line
                        (position.0, position.1 - 1),
                    ));

                    tokens.push(SpannedToken::new(
                        Token::Quote,
                        position,
                        position,
                    ));

                    state = Normal;
                }
                '"' => {
                    tokens.push(SpannedToken::new(
                        Token::Symbol(
                            buffer.clone().iter().collect(),
                        ),
                        mark,
                        // FIXME: this will be bad if we are at the beginning of the line
                        (position.0, position.1 - 1),
                    ));

                    mark = position;
                    buffer.clear();
                    state = InString(false);
                }
                c => buffer.push(c),
            },
            InNumber => match c {
                c if c.is_numeric() => {
                    buffer.push(c);
                }
                '(' => {
                    tokens.push(SpannedToken::new(
                        Token::Number(
                            buffer.clone().iter().collect(),
                        ),
                        mark,
                        // FIXME: this will be bad if we are at the beginning of the line
                        (position.0, position.1 - 1),
                    ));

                    tokens.push(SpannedToken::new(
                        Token::LParen,
                        position,
                        position,
                    ));

                    state = Normal;
                }
                ')' => {
                    tokens.push(SpannedToken::new(
                        Token::Number(
                            buffer.clone().iter().collect(),
                        ),
                        mark,
                        // FIXME: this will be bad if we are at the beginning of the line
                        (position.0, position.1 - 1),
                    ));

                    tokens.push(SpannedToken::new(
                        Token::RParen,
                        position,
                        position,
                    ));

                    state = Normal;
                }
                '\'' => {
                    tokens.push(SpannedToken::new(
                        Token::Number(
                            buffer.clone().iter().collect(),
                        ),
                        mark,
                        // FIXME: this will be bad if we are at the beginning of the line
                        (position.0, position.1 - 1),
                    ));

                    tokens.push(SpannedToken::new(
                        Token::Quote,
                        position,
                        position,
                    ));

                    state = Normal;
                }
                '"' => {
                    tokens.push(SpannedToken::new(
                        Token::Number(
                            buffer.clone().iter().collect(),
                        ),
                        mark,
                        position,
                    ));

                    mark = position;
                    buffer.clear();
                    state = InString(false);
                }
                c => {
                    tokens.push(SpannedToken::new(
                        Token::Number(
                            buffer.clone().iter().collect(),
                        ),
                        mark,
                        // FIXME: this will be bad if we are at the beginning of the line
                        (position.0, position.1 - 1),
                    ));

                    mark = position;
                    buffer.clear();

                    // whitespace bullshit
                    // need to move mark correctly
                    if !c.is_whitespace() {
                        buffer.push(c);
                    }

                    if c.is_whitespace() {
                        state = Normal;
                    } else if c.is_numeric() {
                        state = InNumber;
                    } else {
                        state = InSymbol;
                    }
                }
            },
            InString(escaped) => match c {
                '\\' => {
                    state = InString(true);
                }
                '"' if !escaped => {
                    tokens.push(SpannedToken::new(
                        Token::String(
                            buffer.clone().iter().collect(),
                        ),
                        mark,
                        position,
                    ));
                    state = Normal;
                }
                c => {
                    buffer.push(c);
                    state = InString(false);
                }
            },
        }

        if c == '\n' {
            position.0 += 1;
            position.1 = 0;
        } else {
            position.1 += 1;
        }
    }

    tokens.into_iter()
}

pub fn iter_lex(
    input: impl Iterator<Item = char>,
) -> impl Iterator<Item = SpannedToken> {
    input
        .scan(
            ((1, 0), LexState::Normal),
            |((line, col), state), c| {
                let current_pos = (*line, *col);

                if c == '\n' {
                    *line += 1;
                    *col = 0;
                } else {
                    *col += 1;
                }

                let res = match state {
                    LexState::Normal => match c {
                        '(' => Some(LexItem::Token(
                            SpannedToken::new(
                                Token::LParen,
                                current_pos,
                                current_pos,
                            ),
                        )),
                        ')' => {
                            Some(LexItem::Token(SpannedToken {
                                tok: Token::RParen,
                                start: current_pos,
                                end: current_pos,
                            }))
                        }
                        '\'' => {
                            Some(LexItem::Token(SpannedToken {
                                tok: Token::Quote,
                                start: current_pos,
                                end: current_pos,
                            }))
                        }
                        '"' => {
                            *state = LexState::InString(false);
                            Some(LexItem::StringStart(
                                current_pos,
                            ))
                        }
                        c if c.is_digit(10) => {
                            *state = LexState::InNumber;
                            Some(LexItem::Char(c, current_pos))
                        }
                        c if !c.is_whitespace() => {
                            *state = LexState::InSymbol;
                            Some(LexItem::Char(c, current_pos))
                        }
                        _ => None,
                    },
                    LexState::InString(false) if c == '"' => {
                        *state = LexState::Normal;
                        Some(LexItem::StringEnd)
                    }
                    LexState::InString(is_escaped) => {
                        *state = LexState::InString(
                            c == '\\' && !*is_escaped,
                        );
                        Some(LexItem::Char(c, current_pos))
                    }
                    LexState::InSymbol | LexState::InNumber
                        if c.is_whitespace()
                            || c == '('
                            || c == ')'
                            || c == '\'' =>
                    {
                        *state = LexState::Normal;
                        *col -= 1; // Rewind for the delimiter
                        None
                    }
                    LexState::InSymbol | LexState::InNumber => {
                        Some(LexItem::Char(c, current_pos))
                    }
                };

                Some(res)
            },
        )
        .filter_map(|x| x)
        .peekable()
        .batching(|it| match it.next() {
            Some(LexItem::Token(token)) => Some(token),
            Some(LexItem::StringStart(start)) => {
                let mut content = String::new();
                let mut end = start;

                while let Some(LexItem::Char(c, pos)) = it.next()
                {
                    content.push(c);
                    end = pos;
                }

                if let Some(LexItem::StringEnd) = it.next() {
                    Some(SpannedToken {
                        tok: Token::String(content),
                        start,
                        end,
                    })
                } else {
                    panic!("Unterminated string")
                }
            }
            // All string_ends should be consumed by the match arm above
            Some(LexItem::StringEnd) => {
                unreachable!("You broke it, didn't you?")
            }
            Some(LexItem::Char(first_char, start)) => {
                let mut content = first_char.to_string();
                let mut end = start;

                while let Some(&LexItem::Char(c, pos)) =
                    it.peek()
                {
                    content.push(c);
                    end = pos;
                    it.next();
                }

                let tok = if first_char.is_digit(10) {
                    Token::Number(content)
                } else {
                    Token::Symbol(content)
                };
                Some(SpannedToken { tok, start, end })
            }
            None => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_tokens(
        actual: Vec<SpannedToken>,
        expected: Vec<SpannedToken>,
    ) {
        dbg!(&actual);
        dbg!(&expected);
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
        // compare_tokens(
        //     lex(input.clone()).collect(),
        //     expected.clone(),
        // );
        compare_tokens(iter_lex(input).collect(), expected);
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
                (0, 15),
            ),
            SpannedToken::new(Token::RParen, (0, 16), (0, 16)),
        ];
        compare_tokens(
            lex(input.clone()).collect(),
            expected.clone(),
        );
        compare_tokens(iter_lex(input).collect(), expected);
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
        compare_tokens(
            lex(input.clone()).collect(),
            expected.clone(),
        );
        compare_tokens(iter_lex(input).collect(), expected);
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
                (0, 43),
            ),
            SpannedToken::new(Token::Quote, (0, 45), (0, 45)),
            SpannedToken::new(Token::LParen, (0, 46), (0, 46)),
            SpannedToken::new(Token::RParen, (0, 47), (0, 47)),
            SpannedToken::new(Token::RParen, (0, 48), (0, 48)),
        ];
        compare_tokens(
            lex(input.clone()).collect(),
            expected.clone(),
        );
        compare_tokens(iter_lex(input).collect(), expected);
    }
}
