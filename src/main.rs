mod scanner;
mod parser;

use tiny_ecs::world::World;
use tiny_ecs::systems::Systems;

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

fn reduce_unary_to_negative_literal(ast: &mut World) {
    let unaries = ast.borrow_components::<Unary>().unwrap().to_vec();

    for (id, unary) in unaries.iter().enumerate() {
        if unary.is_none() {
            continue;
        }

        let Unary { operator, operand } = unary.as_ref().unwrap();
        match operator.as_str() {
            "+" => continue,
            "-" => (),
            _ => unreachable!(),
        }

        let operand = match operand {
            ExpressionHandle::Binary(_) => continue,
            ExpressionHandle::Unary(_) => continue,
            ExpressionHandle::Literal(c) => c,
        };

        let operand = operand.borrow().get::<Literal>(ast).unwrap();
        let value = match operand {
            Literal::Integer(value) => Literal::Integer(-value),
            Literal::Float(value) => Literal::Float(-value),
        };

        let entity = ast.new_entity();
        ast.add_component(entity, value);
        ast.repoint_any::<Unary, Literal>(id, entity);
    }
}

fn main() {
    let source = "1 + -2 * 3 / -(+4 - 5)";
    let tokens = scanner::scan(source);

    let mut ast = World::new();
    let expression = parser::parse(&mut ast, &tokens);

    print_expr(&ast, &expression);
    println!();

    let mut systems = Systems::new();
    systems.add_system(reduce_unary_to_negative_literal);
    systems.run(&mut ast);

    print_expr(&ast, &expression);
}
