mod scanner;
mod parser;

use tiny_ecs::world::World;
//use tiny_ecs::systems::Systems;

use parser::ExpressionHandle;
use parser::Unary;
use parser::Literal;
use parser::Binary;

use std::any::TypeId;

enum Expression {
    Binary(Binary),
    Unary(Unary),
    Literal(Literal),
}

fn get_expression(ast: &World, handle: &ExpressionHandle) -> Expression {
    let component = match handle {
        ExpressionHandle::Binary(component) => component,
        ExpressionHandle::Unary(component) => component,
        ExpressionHandle::Literal(component) => component,
    };

    let expr_type = component.borrow().component_type();
    match expr_type {
        _ if expr_type == TypeId::of::<Binary>() => {
            let binary = component.borrow().get::<Binary>(ast).unwrap();
            Expression::Binary(binary.clone())
        }
        _ if expr_type == TypeId::of::<Unary>() => {
            let unary = component.borrow().get::<Unary>(ast).unwrap();
            Expression::Unary(unary.clone())
        }
        _ if expr_type == TypeId::of::<Literal>() => {
            let literal = component.borrow().get::<Literal>(ast).unwrap();
            Expression::Literal(literal.clone())
        }
        _ => unreachable!(),
    }
}

fn print_expr(ast: &World, expr: &ExpressionHandle) {
    let expr = get_expression(ast, expr);
    match expr {
        Expression::Binary(Binary { operator, left, right }) => {
            print!("{{");
            print_expr(ast, &left);
            print!(" {} ", operator);
            print_expr(ast, &right);
            print!("}}");
        }
        Expression::Unary(Unary { operator, operand }) => {
            print!("{{");
            print!("{}, ", operator);
            print_expr(ast, &operand);
            print!("}}");
        }
        Expression::Literal(Literal::Integer(value)) => print!("{}", value),
        Expression::Literal(Literal::Float(value)) => print!("{}", value),
    }
}

fn reduce_unary_to_literal(ast: &mut World) {
    let unaries = ast.borrow_components::<Unary>().unwrap().to_vec();

    for (id, unary) in unaries.iter().enumerate() {
        if unary.is_none() {
            continue;
        }

        let Unary { operator, operand } = unary.as_ref().unwrap();
        match operator.as_str() {
            "+" => (),
            "-" => (),
            _ => unreachable!(),
        }

        let operand = get_expression(ast, operand);
        let operand = match operand {
            Expression::Literal(literal) => literal,
            _ => continue,
        };

        let value = match operand {
            Literal::Integer(value) if operator == "-" => Literal::Integer(-value),
            Literal::Float(value) if operator == "-" => Literal::Float(-value),
            Literal::Integer(value) if operator == "+" => Literal::Integer(value),
            Literal::Float(value) if operator == "+" => Literal::Float(value),
            _ => unreachable!(),
        };

        let entity = ast.new_entity();
        ast.add_component(entity, value);
        ast.repoint_any::<Unary, Literal>(id, entity);
    }
}

fn reduce_binary_to_literal(ast: &mut World) {
    let binaries = ast.borrow_components::<Binary>().unwrap().to_vec();

    for (id, binary) in binaries.iter().enumerate() {
        if binary.is_none() {
            continue;
        }

        let Binary { operator, left, right } = binary.as_ref().unwrap();

        let left = get_expression(ast, left);
        let left = match left {
            Expression::Literal(literal) => literal,
            _ => continue,
        };

        let right = get_expression(ast, right);
        let right = match right {
            Expression::Literal(literal) => literal,
            _ => continue,
        };

        let value = match (left, right) {
            (Literal::Integer(left), Literal::Integer(right)) => match operator.as_str() {
                "+" => Literal::Integer(left + right),
                "-" => Literal::Integer(left - right),
                "*" => Literal::Integer(left * right),
                "/" => Literal::Integer(left / right),
                _ => unreachable!(),
            }
            (Literal::Float(left), Literal::Float(right)) => match operator.as_str() {
                "+" => Literal::Float(left + right),
                "-" => Literal::Float(left - right),
                "*" => Literal::Float(left * right),
                "/" => Literal::Float(left / right),
                _ => unreachable!(),
            }
            (Literal::Integer(left), Literal::Float(right)) => match operator.as_str() {
                "+" => Literal::Float(left as f64 + right),
                "-" => Literal::Float(left as f64 - right),
                "*" => Literal::Float(left as f64 * right),
                "/" => Literal::Float(left as f64 / right),
                _ => unreachable!(),
            }
            (Literal::Float(left), Literal::Integer(right)) => match operator.as_str() {
                "+" => Literal::Float(left + right as f64),
                "-" => Literal::Float(left - right as f64),
                "*" => Literal::Float(left * right as f64),
                "/" => Literal::Float(left / right as f64),
                _ => unreachable!(),
            }
        };

        let entity = ast.new_entity();
        ast.add_component(entity, value);
        ast.repoint_any::<Binary, Literal>(id, entity);
    }
}

macro_rules! named_system {
    ($name:ident) => {
        (stringify!($name), $name as fn(&mut World))
    };
}

fn main() {
    let source = "1 + -2 * 3 / -(+4 - 5)";
    println!("Input: {}", source);
    let tokens = scanner::scan(source);

    let mut ast = World::new();
    let expression = parser::parse(&mut ast, &tokens);

    println!("Parsed into:");
    print!("\t");
    print_expr(&ast, &expression);
    println!();

    let systems = vec![
        named_system!(reduce_unary_to_literal),
        named_system!(reduce_binary_to_literal),
    ];

    for (name, system) in systems {
        system(&mut ast);

        println!("After {}:", name);
        print!("\t");
        print_expr(&ast, &expression);
        println!();
    }
}
