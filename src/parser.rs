use std::slice::Iter;
use std::iter::Peekable;
use tiny_ecs::world::World;
use tiny_ecs::world::ComponentPointer;
use crate::scanner::Token;

macro_rules! operators {
    ($ss:expr) => {
        $ss.iter()
            .map(|s| Token::Operator { operator: s.to_string() })
            .collect::<Vec<_>>()
    };
}

fn consume_either<'a>(
    tokens: &'a mut Peekable<Iter<Token>>,
    types: &[Token],
) -> Option<&'a Token> {
    tokens.next_if(|t| types.iter()
        .any(|ty| {
            let Token::Operator { operator } = ty else { unreachable!() };
            match t {
                Token::Operator { operator: op } => op == operator,
                _ => false,
            }
        })
    )
}

fn consume<'a>(
    tokens: &'a mut Peekable<Iter<Token>>,
    ty: Token,
) -> Option<&'a Token> {
    tokens.next_if(|t| *t == &ty)
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub operator: String,
    pub left: ExpressionPointer,
    pub right: ExpressionPointer,
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: String,
    pub operand: ExpressionPointer,
}

#[derive(Debug, Clone)]
pub enum ExpressionPointer {
    Binary(ComponentPointer<Binary>),
    Unary(ComponentPointer<Unary>),
    Literal(ComponentPointer<Literal>),
}

pub fn parse(ast: &mut World, tokens: &[Token]) -> ExpressionPointer {
    let mut tokens = tokens.iter().peekable();
    expression(ast, &mut tokens).unwrap()
}

fn expression(
    ast: &mut World,
    tokens: &mut Peekable<Iter<Token>>,
) -> Option<ExpressionPointer> {
    term(ast, tokens)
}

fn term(
    ast: &mut World,
    tokens: &mut Peekable<Iter<Token>>,
) -> Option<ExpressionPointer> {
    let mut left = factor(ast, tokens)?;
    let operators = operators!(["+", "-"]);

    while let Some(op) = consume_either(tokens, &operators).cloned() {
        let Token::Operator { operator } = op else { unreachable!() };
        let right = factor(ast, tokens)?;
        let entity = ast.new_entity();
        let expr = ast.add_component(entity,
            Binary {
                operator,
                left,
                right,
            }
        );

        left = ExpressionPointer::Binary(expr);
    }

    Some(left)
}

fn factor(
    ast: &mut World,
    tokens: &mut Peekable<Iter<Token>>,
) -> Option<ExpressionPointer> {
    let mut left = unary(ast, tokens)?;
    let operators = operators!(["*", "/"]);

    while let Some(op) = consume_either(tokens, &operators).cloned() {
        let Token::Operator { operator } = op else { unreachable!() };
        let right = unary(ast, tokens)?;
        let entity = ast.new_entity();
        let expr = ast.add_component(entity,
            Binary {
                operator,
                left,
                right,
            }
        );

        left = ExpressionPointer::Binary(expr);
    }

    Some(left)
}

fn unary(
    ast: &mut World,
    tokens: &mut Peekable<Iter<Token>>,
) -> Option<ExpressionPointer> {
    let operators = operators!(["+", "-"]);

    if let Some(op) = consume_either(tokens, &operators).cloned() {
        let Token::Operator { operator } = op else { unreachable!() };
        let operand = unary(ast, tokens)?;
        let entity = ast.new_entity();
        let expr = ast.add_component(entity,
            Unary {
                operator,
                operand,
            }
        );
        Some(ExpressionPointer::Unary(expr))
    } else {
        primary(ast, tokens)
    }
}

fn primary(
    ast: &mut World,
    tokens: &mut Peekable<Iter<Token>>,
) -> Option<ExpressionPointer> {
    let token = tokens.next()?;
    match token {
        Token::Integer { value } => {
            let entity = ast.new_entity();
            let expr = ast.add_component(entity, Literal::Integer(*value));
            Some(ExpressionPointer::Literal(expr))
        }
        Token::Float { value } => {
            let entity = ast.new_entity();
            let expr = ast.add_component(entity, Literal::Float(*value));
            Some(ExpressionPointer::Literal(expr))
        },
        Token::Grouping { char_: '('  } => {
            let expr = expression(ast, tokens)?;
            consume(tokens, Token::Grouping { char_: ')' })?;
            Some(expr)
        }
        _ => None,
    }
}

