mod json_printer;
mod query_type_printer;
mod source_map_writer;
mod utils;

use graphql_parser::query::{parse_query, ParseError};

fn main() -> Result<(), ParseError> {
    //     let ast = parse_query::<String>(
    //         r#"
    // query {
    //     foo {
    //         bar
    //         baz
    //     }
    // }
    // "#,
    //     )?
    //     .to_owned();
    let ast = parse_query::<String>(
        r#"
fragment A on B {
    foo {
        bar
        baz
    }
}
"#,
    )?;
    // Format canonical representation
    println!("{ast}");

    // let mut res = String::new();
    // print_query(&ast, &mut res);
    // println!("{res}");

    // let mut res = String::new();
    // let printer = QueryTypePrinter::new(QueryTypePrinterOptions::default(), &mut res);

    Ok(())
}
