
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Grouping { char_: char },
    Integer { value: i64 },
    Float { value: f64 },
    Identifier { name: String },
    Operator { operator: String },
}

pub fn scan(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();

    while chars.peek().is_some() {
        let token = scan_one(&mut chars);
        tokens.push(token);
    }

    tokens
}

fn scan_one(chars: &mut Peekable<Chars>) -> Token {
    let c = chars.next().unwrap();
    match c {
        'a'..='z' |
        'A'..='Z' |
        '_' => scan_identifier(c, chars),

        '+' | '-' |
        '*' | '/' |
        '%' | '^' => Token::Operator { operator: c.to_string() },
        '(' | ')' => Token::Grouping { char_: c },
        '0'..='9' => scan_number(c, chars),

        ' ' | '\t' | '\r' | '\n' => scan_one(chars),
        _ => panic!("Unexpected character: {}", c),
    }
}

fn scan_number(current: char, chars: &mut Peekable<Chars>) -> Token {
    let number = &mut current.to_string();
    while let Some(&c) = chars.peek() {
        if !c.is_ascii_digit() && c != '.' && c != '_'{
            break;
        }

        if c != '_' {
            number.push(c);
        }

        chars.next();
    }

    let integer = number.parse::<i64>();
    match integer {
        Ok(value) => Token::Integer { value },
        Err(_) => number.parse::<f64>()
            .map(|value| Token::Float { value })
            .unwrap_or_else(|_| panic!("Invalid number: {}", number)),
    }
}

fn scan_identifier(current: char, chars: &mut Peekable<Chars>) -> Token {
    let identifier = &mut current.to_string();
    let first = identifier
        .chars().next()
        .unwrap_or_else(|| {
            unreachable!("There must be at least one character")
        });

    if !first.is_alphabetic() && first != '_' {
        unreachable!("First character must be alphabetic or underscore")
    }

    while let Some(&c) = chars.peek() {
        if !c.is_ascii_alphanumeric() && c != '_' {
            break;
        }

        identifier.push(c);
        chars.next();
    }

    Token::Identifier { name: identifier.to_string() }
}
