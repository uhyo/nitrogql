use std::borrow::Cow;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use log::debug;
use nitrogql_semantics::{ast_to_type_system, type_system_to_ast};

use crate::context::LoadedSchema;
use crate::error::CliError;
use crate::file_store::{FileKind, FileStore};
use nitrogql_config_file::GenerateMode;
use nitrogql_error::Result;
use nitrogql_printer::{
    print_types_for_operation_document, OperationTypePrinterOptions, SchemaTypePrinter,
    SchemaTypePrinterOptions,
};
use nitrogql_utils::relative_path;
use sourcemap_writer::{print_source_map_json, SourceWriter, SourceWriterBuffers};

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
            file_store,
            ..
        } => {
            let Some(ref schema_output) = config.config.generate.schema_output else {
                return Err(CliError::OptionRequired { option: String::from("schemaOutput"), command: String::from("generate") }.into())
            };
            let schema_output = config.root_dir.join(schema_output);
            {
                debug!("Processing schema");
                let file_map = FileMap {
                    file_store,
                    file_indices: file_store
                        .iter()
                        .map(|(idx, (_, _, kind))| {
                            if kind == FileKind::Schema {
                                idx
                            } else {
                                usize::MAX
                            }
                        })
                        .collect(),
                };

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
                writer.set_file_index_mapper(file_map.file_indices.clone());
                let mut printer = SchemaTypePrinter::new(options, &mut writer);

                match schema {
                    LoadedSchema::GraphQL(schema) => {
                        printer.print_document(schema)?;
                    }
                    LoadedSchema::Introspection(schema) => {
                        let ast = type_system_to_ast(schema);
                        printer.print_document(&ast)?;
                    }
                }

                let buffers = writer.into_buffers();
                write_file_and_sourcemap(&file_map, &schema_output, buffers)?;
            }

            let schema = schema.map_into(|doc| Cow::Owned(ast_to_type_system(doc)), Cow::Borrowed);

            for (path, doc, file_index) in operations.iter() {
                debug!("Processing {}", path.to_string_lossy());
                let file_map = FileMap {
                    file_store,
                    file_indices: file_store
                        .iter()
                        .map(|(idx, (_, _, kind))| {
                            if kind == FileKind::Schema {
                                idx
                            } else if idx == *file_index {
                                file_store.schema_len()
                            } else {
                                usize::MAX
                            }
                        })
                        .collect(),
                };

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
                writer.set_file_index_mapper(file_map.file_indices.clone());
                let mut printer_options = OperationTypePrinterOptions::default();
                if config.config.generate.mode == GenerateMode::StandaloneTS4_0 {
                    printer_options.print_values = true;
                }
                // Todo custom schema_root_types
                printer_options.schema_source =
                    path_to_ts(relative_path(&decl_file_path, &schema_output))
                        .to_string_lossy()
                        .to_string();
                print_types_for_operation_document(printer_options, &schema, doc, &mut writer);

                let buffers = writer.into_buffers();

                write_file_and_sourcemap(&file_map, &decl_file_path, buffers)?;
            }
            eprintln!("'generate' finished");
            Ok(context)
        }
    }
}

#[derive(Debug)]
struct FileMap<'src> {
    pub file_store: &'src FileStore,
    /// Mapping from file index in file_store to source map index.
    pub file_indices: Vec<usize>,
}

fn write_file_and_sourcemap(
    file_map: &FileMap,
    output_file_path: &Path,
    buffers: SourceWriterBuffers,
) -> Result<()> {
    let source_files = file_map
        .file_indices
        .iter()
        .zip(file_map.file_store.iter())
        .filter_map(|(idx, (_, (path, _, _)))| {
            if *idx == usize::MAX {
                return None;
            }

            Some(path)
        })
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
