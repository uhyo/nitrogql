mod base64_vlq;
mod json_printer;
mod query_type_printer;
mod source_map_writer;
mod utils;

use graphql_parser::query::{parse_query, ParseError};

use crate::{
    query_type_printer::{QueryTypePrinter, QueryTypePrinterOptions},
    source_map_writer::source_writer::{print_source_map_json, SourceWriter},
};

fn main() -> Result<(), ParseError> {
    let ast = parse_query::<String>(
        r#"
    query sampleQuery {
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

    let mut writer = SourceWriter::new();
    let mut printer = QueryTypePrinter::new(QueryTypePrinterOptions::default(), &mut writer);
    printer.print_document(&ast);

    let buffers = writer.into_buffers();

    println!("{}", buffers.buffer);

    let mut source_map = String::new();
    print_source_map_json(
        "file.d.ts",
        "source.graphql",
        &buffers.names,
        &buffers.source_map,
        &mut source_map,
    );

    println!("{}", source_map);

    Ok(())
}
