use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use log::debug;

use crate::error::CliError;
use nitrogql_config_file::GenerateMode;
use nitrogql_error::Result;
use nitrogql_printer::{
    print_types_for_operation_document, OperationTypePrinterOptions, SchemaTypePrinter,
    SchemaTypePrinterOptions,
};
use nitrogql_utils::relative_path;
use sourcemap_writer::{print_source_map_json, SourceWriter, SourceWriterBuffers};

use super::context::FileByIndex;
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
            let Some(ref schema_output) = config.config.generate.schema_output else {
                return Err(CliError::OptionRequired { option: String::from("schema-output"), command: String::from("generate") }.into())
            };
            let schema_output = config.root_dir.join(schema_output);
            {
                debug!("Processing schema");
                let mut options = SchemaTypePrinterOptions::default();
                options.scalar_types.extend(
                    config
                        .config
                        .generate
                        .scalar_types
                        .iter()
                        .map(|(key, value)| (key.to_owned(), value.to_owned())),
                );
                let mut writer = SourceWriter::new();
                let mut printer = SchemaTypePrinter::new(options, &mut writer);

                printer.print_document(schema)?;

                let buffers = writer.into_buffers();
                write_file_and_sourcemap(file_by_index, &schema_output, buffers)?;
            }

            for (path, doc, file_by_index) in operations.iter() {
                debug!("Processing {}", path.to_string_lossy());
                let decl_file_path = {
                    let mut path = path.clone();
                    path.set_extension(match config.config.generate.mode {
                        GenerateMode::WithLoaderTS5_0 => "d.graphql.ts",
                        GenerateMode::WithLoaderTS4_0 => "graphql.d.ts",
                        GenerateMode::StandaloneTS4_0 => "graphql.ts",
                    });
                    path
                };

                let mut writer = SourceWriter::new();
                let mut printer_options = OperationTypePrinterOptions::default();
                if config.config.generate.mode == GenerateMode::StandaloneTS4_0 {
                    printer_options.print_values = true;
                }
                // Todo custom schema_root_types
                printer_options.schema_source =
                    path_to_ts(relative_path(&decl_file_path, &schema_output))
                        .to_string_lossy()
                        .to_string();
                print_types_for_operation_document(printer_options, schema, &doc, &mut writer);

                let buffers = writer.into_buffers();

                write_file_and_sourcemap(file_by_index, &decl_file_path, buffers)?;
            }
            eprintln!("'generate' finished");
            Ok(context)
        }
    }
}

fn write_file_and_sourcemap(
    file_by_index: &FileByIndex,
    output_file_path: &Path,
    buffers: SourceWriterBuffers,
) -> Result<()> {
    let source_files = file_by_index
        .iter()
        .map(|(path, _)| path.as_path())
        .collect::<Vec<_>>();
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

    debug!("Writing {}", output_file_path.to_string_lossy());
    {
        let parent_dir = output_file_path.parent();
        if let Some(parent_dir) = parent_dir {
            fs::create_dir_all(parent_dir)?;
        }
    }
    let mut output_file = File::create(output_file_path)?;

    writeln!(&mut output_file, "{}", &buffers.buffer)?;
    writeln!(
        &mut output_file,
        "//# sourceMappingURL={}",
        source_map_file_path.file_name().unwrap().to_string_lossy()
    )?;

    let mut source_map = String::new();
    print_source_map_json(
        output_file_path,
        &source_files,
        &buffers.names,
        &buffers.source_map,
        &mut source_map,
    )?;

    debug!("Writing {}", source_map_file_path.to_string_lossy());
    fs::write(source_map_file_path, &source_map)?;
    Ok(())
}

/// Removes '.d.ts' suffix
fn path_to_ts(mut path: PathBuf) -> PathBuf {
    match path.file_name() {
        None => path,
        Some(file_name) => match file_name.to_os_string().into_string() {
            Err(_) => path,
            Ok(mut file_name) => {
                if file_name.ends_with(".d.ts") {
                    file_name.truncate(file_name.len() - 5);
                    path.set_file_name(file_name);
                    return path;
                }
                if file_name.ends_with(".ts") {
                    file_name.truncate(file_name.len() - 3);
                    path.set_file_name(file_name);
                    return path;
                }
                path
            }
        },
    }
}
