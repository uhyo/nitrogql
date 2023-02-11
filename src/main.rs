mod printer;

use graphql_parser::query::{parse_query, ParseError};

use crate::printer::print_query;

fn main() -> Result<(), ParseError> {
    let ast = parse_query::<String>(
        r#"
query {
    foo {
        bar
        baz
    }
}
"#,
    )?
    .to_owned();
    // Format canonical representation
    println!("{ast}");

    let mut res = String::new();
    print_query(&ast, &mut res);
    println!("{res}");

    Ok(())
}
