use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use log::debug;

use crate::cli::error::CliError;
use crate::source_map_writer::source_writer::SourceWriterBuffers;
use crate::type_printer::schema_type_printer::printer::{
    SchemaTypePrinter, SchemaTypePrinterOptions,
};
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
        CliContext::SchemaResolved {
            ref schema,
            ref operations,
            ref config,
            ref file_by_index,
            ..
        } => {
            let Some(ref schema_output) = config.schema_output else {
                return Err(CliError::OptionRequired { option: String::from("schema-output"), command: String::from("generate") }.into())
            };
            let source_files = file_by_index
                .iter()
                .map(|(path, _)| path.to_string_lossy())
                .collect::<Vec<_>>();
            let source_files = source_files
                .iter()
                .map(|path| path.as_ref())
                .collect::<Vec<_>>();
            {
                debug!("Processing schema");
                let mut writer = SourceWriter::new();
                let mut printer =
                    SchemaTypePrinter::new(SchemaTypePrinterOptions::default(), &mut writer);

                printer.print_document(schema)?;

                let buffers = writer.into_buffers();
                write_file_and_sourcemap(&source_files, &schema_output, buffers)?;
            }

            for (path, doc) in operations.iter() {
                debug!("Processing {}", path.to_string_lossy());
                let decl_file_path = {
                    let mut path = path.clone();
                    path.set_extension("d.graphql.ts");
                    path
                };

                let mut writer = SourceWriter::new();
                let mut printer =
                    QueryTypePrinter::new(QueryTypePrinterOptions::default(), &mut writer);
                printer.print_document(&doc);

                let buffers = writer.into_buffers();

                write_file_and_sourcemap(&source_files, &decl_file_path, buffers)?;
            }
            eprintln!("'generate' succeeded without errors");
            Ok(context)
        }
    }
}

fn write_file_and_sourcemap(
    source_files: &[&str],
    output_file_path: &Path,
    buffers: SourceWriterBuffers,
) -> Result<()> {
    let source_map_file_path = {
        let mut path = output_file_path.to_owned();
        match path.file_name() {
            None => {
                return Err(
                    CliError::FailedToCalculateSourceMapFileName { path: path.clone() }.into(),
                );
            }
            Some(file_name) => {
                let mut file_name = file_name.to_owned();
                file_name.push(".map");
                path.set_file_name(file_name);
            }
        }
        path
    };
    let mut output_file = File::create(output_file_path)?;

    debug!("Writing {}", output_file_path.to_string_lossy());
    writeln!(&mut output_file, "{}", &buffers.buffer)?;
    writeln!(
        &mut output_file,
        "//# sourceMappingURL={}",
        source_map_file_path.file_name().unwrap().to_string_lossy()
    )?;

    let mut source_map = String::new();
    print_source_map_json(
        &output_file_path.to_string_lossy(),
        source_files,
        &buffers.names,
        &buffers.source_map,
        &mut source_map,
    );

    debug!("Writing {}", source_map_file_path.to_string_lossy());
    fs::write(source_map_file_path, &source_map)?;
    Ok(())
}
