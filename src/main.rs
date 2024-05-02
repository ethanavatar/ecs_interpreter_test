mod scanner;
mod parser;

use tiny_ecs::world::World;
use tiny_ecs::systems::Systems;
use tiny_ecs::world::ComponentPointer;

use parser::ExpressionPointer;
use parser::Binary;
use parser::Unary;
use parser::Literal;

fn print_expr(ast: &World, expr: ExpressionPointer) {
    match expr {
        ExpressionPointer::Binary(component) => {
            let binary = component.get(ast).unwrap();
            print!("{{");
            print_expr(ast, binary.left);
            print!(" {} ", binary.operator);
            print_expr(ast, binary.right);
            print!("}}");
        }
        ExpressionPointer::Unary(component) => {
            let unary = component.get(ast).unwrap();
            print!("{{");
            print!("{}", unary.operator);
            print_expr(ast, unary.operand);
            print!("}}");
        }
        ExpressionPointer::Literal(component) => {
            let literal = component.get(ast).unwrap();
            match literal {
                Literal::Integer(value) => print!("{}", value),
                Literal::Float(value) => print!("{}", value),
            }
        }
    }
}

fn reduce_literals_with_unaries(ast: &mut World) {
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
            ExpressionPointer::Binary(_) => continue,
            ExpressionPointer::Unary(_) => continue,
            ExpressionPointer::Literal(c) => c.clone(),
        };

        let operand = operand.get(ast).unwrap();
        let value = match operand {
            Literal::Integer(value) => Literal::Integer(-value),
            Literal::Float(value) => Literal::Float(-value),
        };

        let entity = ast.new_entity();
        let component = ast.add_component(entity, value);

        println!("Replacing unary at {} with literal at {}", id, entity);
        ast.replace_component(id, component);
    }
}

fn main() {
    let source = "1 + -2 * 3 / -(+4 - 5)";
    let tokens = scanner::scan(source);

    let mut ast = World::new();
    let expression = parser::parse(&mut ast, &tokens);

    print_expr(&ast, expression.clone());
    println!();

    let mut systems = Systems::new();
    systems.add_system(reduce_literals_with_unaries);
    systems.run(&mut ast);

    print_expr(&ast, expression);
}
