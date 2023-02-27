mod base64_vlq;
mod extension_resolver;
mod graphql_parser;
mod graphql_printer;
mod json_printer;
mod source_map_writer;
mod type_printer;
mod type_system_validator;
mod utils;

use std::fs::{self, File};
use std::io::Write;

use crate::graphql_parser::parser::parse_operation_document;
use glob::glob;

use crate::{
    source_map_writer::source_writer::{print_source_map_json, SourceWriter},
    type_printer::query_type_printer::{QueryTypePrinter, QueryTypePrinterOptions},
};

fn main() -> anyhow::Result<()> {
    //     println!(
    //         "{:?}",
    //         parse_operation_document(
    //             "
    // query sample($foo: Int! =3) @a(foo: A)
    // {
    //     foo
    //     bar:baz
    //     ... one
    //     ... on A {
    //         abc
    //     }
    // }",
    //         )?
    //     );

    let target_files = glob("./**/*.graphql")?;
    for path in target_files {
        let path = path?;
        let decl_file_path = {
            let mut path = path.clone();
            path.set_extension("d.graphql.ts");
            path
        };
        let source_map_file_path = {
            let mut path = path.clone();
            path.set_extension("d.graphql.ts.map");
            path
        };
        let input_file_name = path.file_name().unwrap();
        let decl_file_name = decl_file_path.file_name().unwrap();

        let file_content = fs::read_to_string(&path)?;
        let ast = parse_operation_document(&file_content)?;

        let mut writer = SourceWriter::new();
        let mut printer = QueryTypePrinter::new(QueryTypePrinterOptions::default(), &mut writer);
        printer.print_document(&ast);

        let buffers = writer.into_buffers();

        // fs::write(&decl_file_path, &buffers.buffer)?;

        let mut decl_file = File::create(&decl_file_path)?;
        let decl_file_name_str = decl_file_name.to_string_lossy().into_owned();

        writeln!(&mut decl_file, "{}", &buffers.buffer)?;
        writeln!(
            &mut decl_file,
            "//# sourceMappingURL={}",
            source_map_file_path.file_name().unwrap().to_string_lossy()
        )?;

        let mut source_map = String::new();
        print_source_map_json(
            &decl_file_name_str,
            &input_file_name.to_string_lossy(),
            &buffers.names,
            &buffers.source_map,
            &mut source_map,
        );

        fs::write(source_map_file_path, &source_map)?;
    }
    Ok(())
}
