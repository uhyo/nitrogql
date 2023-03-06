use std::fs::{self, File};
use std::io::Write;

use log::debug;

use crate::{
    error::Result,
    source_map_writer::source_writer::{print_source_map_json, SourceWriter},
    type_printer::query_type_printer::{QueryTypePrinter, QueryTypePrinterOptions},
};

use super::{check::run_check, context::CliContext};

pub fn run_generate(mut context: CliContext) -> Result<CliContext> {
    if let CliContext::SchemaUnresolved { .. } = context {
        // Seems like check is not run
        context = run_check(context)?;
    }
    match context {
        CliContext::SchemaUnresolved { .. } => panic!("Something went wrong"),
        CliContext::SchemaResolved { ref operations, .. } => {
            for (path, doc) in operations.iter() {
                debug!("Processing {}", path.to_string_lossy());
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

                let mut writer = SourceWriter::new();
                let mut printer =
                    QueryTypePrinter::new(QueryTypePrinterOptions::default(), &mut writer);
                printer.print_document(&doc);

                let buffers = writer.into_buffers();

                debug!("Writing {}", decl_file_path.to_string_lossy());
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

                debug!("Writing {}", source_map_file_path.to_string_lossy());
                fs::write(source_map_file_path, &source_map)?;
            }
            eprintln!("'generate' succeeded without errors");
            Ok(context)
        }
    }
}
