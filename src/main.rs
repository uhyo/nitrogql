mod json_printer;
mod query_type_printer;
mod source_map_writer;
mod utils;

use graphql_parser::query::{parse_query, ParseError};

use crate::{
    query_type_printer::{QueryTypePrinter, QueryTypePrinterOptions},
    source_map_writer::just_writer::JustWriter,
};

fn main() -> Result<(), ParseError> {
    let ast = parse_query::<String>(
        r#"
    query {
        foo {
            bar
            baz
            ...U
        }
    }

    fragment U on Foo {
        abc
        def
    }
    "#,
    )?
    .to_owned();
    //     let ast = parse_query::<String>(
    //         r#"
    // fragment A on B {
    //     foo {
    //         bar
    //         baz
    //     }
    // }
    // "#,
    //     )?;
    // Format canonical representation
    println!("{ast}");

    // let mut res = String::new();
    // print_query(&ast, &mut res);
    // println!("{res}");

    let mut res = String::new();
    let mut writer = JustWriter::new(&mut res);
    let mut printer = QueryTypePrinter::new(QueryTypePrinterOptions::default(), &mut writer);
    printer.print_document(&ast);

    println!("{res}");

    Ok(())
}
