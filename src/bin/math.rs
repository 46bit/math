extern crate advent;
#[macro_use]
extern crate nom;

use advent::math;

fn main() {
    //println!("name {:?}", variable_name(b"a"));
    //println!("name {:?}", variable_name(b"abc"));
    //println!("name {:?}", variable_name(b"abc "));
    //println!("name {:?}", variable_name(b"abc="));
    //println!("name {:?}", variable_name(b"abc ="));
    //println!("");

    //println!("expression {:?}", expression(b"123456"));
    //println!("expression {:?}", expression(b"1;"));
    //println!("expression {:?}", expression(b"123;"));
    //println!("expression {:?}", expression(b"1+4+7;"));
    //println!("expression {:?}", expression(b"1 + 4 + 7;"));
    //println!("expression {:?}", expression(b"3 + 6 + 9 ;"));
    //println!("expression {:?}", expression(b"1 + 3 * 6 / 9;"));
    //println!("expression {:?}", expression(b"1 + 4 + 7"));
    //println!("expression {:?}", expression(b"1 + 4 + a(5)"));
    //println!("expression {:?}", expression(b"1 + 4 + a(5 + 5)"));
    //println!("");

    //println!("variable_assignment {:?}", variable_assignment(b"a=1;"));
    //println!("variable_assignment {:?}", variable_assignment(b"a = 1;"));
    //println!("variable_assignment {:?}", variable_assignment(b"a = 1 + 3 * 6 / 9;"));
    //println!("");

    //println!("function_definition {:?}", function_definition(b"a(b) = b;"));
    //println!("");

    //println!("statement {:?}", statement(b"a = 1 + 3 * 6 / 9;"));
    //println!("statement {:?}", statement(b"a(b) = b;"));
    //println!("");

    println!(
        "statements {:?}",
        math::parser::statements(b"a = 1 - -3 * 6 / 9;\nb = 5 * -3;")
    );
    println!(
        "statements {:?}",
        math::parser::statements(b"a = 1 + 3 * 6 / 9;\nb = 5 * a(-5 * 3) + 5;")
    );
    println!(
        "statements {:?}",
        math::parser::statements(
            b"a = 1 + a * 6 / 9;\nb = 5 * a(5 * -3) + 5;\na(i, j) = 5 * i - j;"
        )
    );
    println!("");

    let mut executor = math::interpreter::Executor::new();
    let statements = math::parser::statements(
        b"a = 1 + 89 / 9;\na(i, j) = 5 * i - j;\nb = 5 * a(5 * -3, 2) + 5;",
    ).unwrap()
        .1;
    executor.run(statements);
    println!("{:?}", executor.variables);

    println!(
        "{:?}",
        math::interpret(b"a = 1 + 89 / 9;\na(i, j) = 5 * i - j;\nb = 5 * a(5 * -3, 2) + 5;")
    );
}
