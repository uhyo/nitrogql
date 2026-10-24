use std::{
    collections::{HashMap, HashSet},
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use nitrogql_ast::{OperationDocument, base::Pos, set_current_file_of_pos};
use nitrogql_checker::{OperationCheckContext, check_operation_document};
use nitrogql_error::PositionedError;
use nitrogql_parser::parse_operation_document;
use nitrogql_printer::{
    OperationTypePrinterOptions, SchemaTypePrinter, SchemaTypePrinterOptions,
    print_fallback_types_for_operation_document, print_types_for_operation_document_with_fallback,
};
use nitrogql_semantics::{
    OperationExtension, OperationResolver, resolve_operation_extensions, resolve_operation_imports,
};
use nitrogql_utils::{normalize_path, relative_path, resolve_relative_path};
use sourcemap_writer::{RecordedSpan, SpanWriter};
use thiserror::Error;

use crate::{generate::path_to_ts, schema_loader::LoadedSchema};

use super::{
    fallback::fallback_definitions,
    position::{LineIndex, encode_offset},
    project::{LoadedProjectSchema, Project},
    protocol::{
        MapperDiagnostic, PositionEncoding, SpanMapKind, SpanMapping, TransformResult, codes,
    },
};

/// Virtual content for files that do not export anything.
const EMPTY_MODULE: &str = "export {};\n";

#[derive(Debug, Error)]
enum TransformError {
    #[error("Failed to read '{path}': {message}")]
    ReadFile { path: String, message: String },
    #[error(
        "Cannot determine where to import schema types from. Set 'schemaOutput' or 'schemaModuleSpecifier' option in nitrogql config, or use GraphQL files as schema."
    )]
    NoSchemaSource,
}

/// Transforms given file into TypeScript code.
pub fn transform(
    project: &Project,
    file_name: &Path,
    content: &str,
    encoding: PositionEncoding,
) -> TransformResult {
    let file_name = normalize_path(file_name);
    if let Some(file_index) = project.schema_file_index(&file_name) {
        return transform_schema_file(project, file_index, content, encoding);
    }
    transform_operation_file(project, &file_name, content, encoding)
}

/// Transforms a schema file.
/// Schema types are emitted from the first GraphQL schema file, so that operation files can import them.
fn transform_schema_file(
    project: &Project,
    file_index: usize,
    content: &str,
    encoding: PositionEncoding,
) -> TransformResult {
    let (_, loaded_content, _) = &project.file_store[file_index];
    // Positions are valid only if content is the same as the one loaded.
    let is_same_content = *loaded_content == content;
    let index = LineIndex::new(content, encoding);
    let files = |idx: usize| project.file_store.get_file(idx).map(|(path, _, _)| path);
    let mut result = TransformResult {
        text: EMPTY_MODULE.to_owned(),
        extension: ".ts",
        mappings: vec![],
        diagnostics: vec![],
    };

    let schema = match project.schema() {
        Ok(schema) => schema,
        Err(errors) => {
            if is_same_content {
                let errors_in_file = errors
                    .iter()
                    .filter(|err| err.position().is_some_and(|pos| pos.file == file_index))
                    .collect::<Vec<_>>();
                result.diagnostics = to_diagnostics(
                    errors_in_file,
                    file_index,
                    &files,
                    &index,
                    codes::VALIDATION_ERROR,
                );
            }
            return result;
        }
    };
    if schema_module_file(project, schema).map(|(_, idx)| *idx) != Some(file_index) {
        return result;
    }
    let LoadedSchema::GraphQL(ref document) = schema.schema else {
        return result;
    };

    let mut options = SchemaTypePrinterOptions::from_config(&project.config);
    // Content-mapped files only provide types.
    options.emit_schema_runtime = false;
    let mut writer = SpanWriter::new();
    let mut printer = SchemaTypePrinter::new(options, &mut writer);
    if let Err(err) = printer.print_document(document) {
        result.diagnostics = to_diagnostics(
            [&PositionedError::from(err)],
            file_index,
            &files,
            &index,
            codes::VALIDATION_ERROR,
        );
        return result;
    }
    let (text, spans) = writer.into_parts();
    if is_same_content {
        result.mappings = to_mappings(&spans, file_index, &text, &index, encoding);
    }
    result.text = text;
    result
}

/// Transforms an operation file into type definitions.
///
/// If the file has errors but can be parsed, a fallback module is emitted along with diagnostics.
/// It has the same exports as usual but types are `any`, so that importing files keep working.
/// Validation errors only make the affected definitions `any`;
/// other errors (e.g. in the project or in imports) affect the whole file.
fn transform_operation_file(
    project: &Project,
    file_name: &Path,
    content: &str,
    encoding: PositionEncoding,
) -> TransformResult {
    // Operation files are numbered after schema files.
    let root_index = project.file_store.schema_len();
    let files = load_operation_files(file_name, content, root_index);
    let file_path = |idx: usize| {
        if idx < root_index {
            project.file_store.get_file(idx).map(|(path, _, _)| path)
        } else {
            files.files.get(idx - root_index).map(|(path, _)| path)
        }
    };
    let index = LineIndex::new(content, encoding);

    let mut documents = HashMap::new();
    for (idx, (path, source)) in files.files.iter().enumerate() {
        set_current_file_of_pos(root_index + idx);
        // Errors are reported by load_operation_files.
        let Ok(document) = parse_operation_document(source) else {
            continue;
        };
        let Ok(document) = resolve_operation_extensions(document) else {
            continue;
        };
        documents.insert(path.as_path(), document);
    }

    let error_result = |diagnostics: Vec<MapperDiagnostic>| {
        let Some((document, _)) = documents.get(file_name) else {
            return TransformResult {
                text: EMPTY_MODULE.to_owned(),
                extension: ".ts",
                mappings: vec![],
                diagnostics,
            };
        };
        let mut options = OperationTypePrinterOptions::from_config(&project.config);
        options.print_values = false;
        let mut writer = SpanWriter::new();
        print_fallback_types_for_operation_document(options, document, &mut writer);
        let (text, spans) = writer.into_parts();
        TransformResult {
            mappings: to_mappings(&spans, root_index, &text, &index, encoding),
            text,
            extension: ".ts",
            diagnostics,
        }
    };
    let diagnose = |errors: &[PositionedError], code: u32| {
        to_diagnostics(errors, root_index, &file_path, &index, code)
    };

    if !files.syntax_errors.is_empty() {
        return error_result(diagnose(&files.syntax_errors, codes::SYNTAX_ERROR));
    }
    if !files.errors.is_empty() {
        return error_result(diagnose(&files.errors, codes::VALIDATION_ERROR));
    }
    let schema = match project.schema() {
        Ok(schema) => schema,
        Err(errors) => {
            let mut diagnostics = diagnose(errors, codes::PROJECT_ERROR);
            for diagnostic in diagnostics.iter_mut() {
                diagnostic.message_text = format!(
                    "Failed to load nitrogql project: {}",
                    diagnostic.message_text
                );
            }
            return error_result(diagnostics);
        }
    };

    let Some((document, extension)) = documents.get(file_name) else {
        return error_result(vec![]);
    };
    let document = match resolve_operation_imports(
        (file_name, document, extension),
        &DocumentResolver(&documents),
    ) {
        Ok(document) => document,
        Err(err) => return error_result(diagnose(&[err.into()], codes::VALIDATION_ERROR)),
    };

    let check_errors =
        check_operation_document(&document, &OperationCheckContext::new(&schema.type_system))
            .into_iter()
            .map(PositionedError::from)
            .collect::<Vec<_>>();
    let mut diagnostics = diagnose(&check_errors, codes::VALIDATION_ERROR);
    // Definitions with errors get `any` types, so that other definitions keep their types.
    let Some(fallback) = fallback_definitions(&document, &check_errors) else {
        return error_result(diagnostics);
    };

    let Some(schema_source) = schema_source(project, schema, file_name) else {
        diagnostics.extend(diagnose(
            &[TransformError::NoSchemaSource.into()],
            codes::PROJECT_ERROR,
        ));
        return error_result(diagnostics);
    };
    let mut options = OperationTypePrinterOptions::from_config(&project.config);
    // Runtime values are provided by loaders, not by content-mapped files.
    options.print_values = false;
    options.schema_source = schema_source;

    let mut writer = SpanWriter::new();
    let printer_errors = print_types_for_operation_document_with_fallback(
        options,
        &schema.type_system,
        &document,
        |pos| fallback.contains(pos),
        &mut writer,
    );
    // Printer errors mean that the checker has missed an error.
    // The same error is reported for every definition that spreads the erroneous fragment.
    let mut seen = HashSet::new();
    let printer_errors = printer_errors
        .into_iter()
        .filter(|err| seen.insert((err.position, err.message.to_string())))
        .map(PositionedError::from)
        .collect::<Vec<_>>();
    diagnostics.extend(diagnose(&printer_errors, codes::VALIDATION_ERROR));
    let (text, spans) = writer.into_parts();
    TransformResult {
        mappings: to_mappings(&spans, root_index, &text, &index, encoding),
        text,
        extension: ".ts",
        diagnostics,
    }
}

struct OperationFiles {
    /// Loaded files. The first one is the transformed file, followed by imported files.
    files: Vec<(PathBuf, String)>,
    syntax_errors: Vec<PositionedError>,
    errors: Vec<PositionedError>,
}

/// Loads given operation file and files imported from it.
fn load_operation_files(file_name: &Path, content: &str, root_index: usize) -> OperationFiles {
    let mut result = OperationFiles {
        files: vec![(file_name.to_owned(), content.to_owned())],
        syntax_errors: vec![],
        errors: vec![],
    };
    let mut idx = 0;
    while idx < result.files.len() {
        let (path, source) = &result.files[idx];
        set_current_file_of_pos(root_index + idx);
        let imports = match parse_operation_document(source) {
            Err(err) => {
                result.syntax_errors.push(err.into());
                vec![]
            }
            Ok(document) => match resolve_operation_extensions(document) {
                Err(err) => {
                    result.errors.push(err.into());
                    vec![]
                }
                Ok((_, extension)) => extension
                    .imports
                    .iter()
                    .map(|import| {
                        let imported = normalize_path(&resolve_relative_path(
                            path,
                            Path::new(import.path.value.as_str()),
                        ));
                        (imported, import.path.position)
                    })
                    .collect(),
            },
        };
        for (imported, position) in imports {
            if result.files.iter().any(|(path, _)| *path == imported) {
                continue;
            }
            match fs::read_to_string(&imported) {
                Ok(source) => result.files.push((imported, source)),
                Err(err) => result.errors.push(PositionedError::new(
                    TransformError::ReadFile {
                        path: imported.to_string_lossy().into_owned(),
                        message: match err.kind() {
                            ErrorKind::NotFound => "file not found".to_owned(),
                            _ => err.to_string(),
                        },
                    }
                    .into(),
                    Some(position),
                    vec![],
                )),
            }
        }
        idx += 1;
    }
    result
}

struct DocumentResolver<'a, 'src>(
    &'a HashMap<&'a Path, (OperationDocument<'src>, OperationExtension<'src>)>,
);

impl<'src> OperationResolver<'src> for DocumentResolver<'_, 'src> {
    fn resolve(
        &self,
        path: &Path,
    ) -> Option<(&OperationDocument<'src>, &OperationExtension<'src>)> {
        self.0
            .get(normalize_path(path).as_path())
            .map(|(document, extension)| (document, extension))
    }
}

/// Returns the schema file that provides schema types, along with its index in the file store.
/// This is used only if schema types are not provided by other means.
fn schema_module_file<'a>(
    project: &'a Project,
    schema: &LoadedProjectSchema,
) -> Option<&'a (PathBuf, usize)> {
    let generate = &project.config.generate;
    if generate.schema_output.is_some() || generate.schema_module_specifier.is_some() {
        return None;
    }
    if !matches!(schema.schema, LoadedSchema::GraphQL(_)) {
        return None;
    }
    project.schema_files.iter().find(|(path, _)| {
        !matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("json" | "js" | "mjs" | "cjs" | "ts" | "mts" | "cts")
        )
    })
}

/// Determines the module specifier to import schema types from.
fn schema_source(
    project: &Project,
    schema: &LoadedProjectSchema,
    file_name: &Path,
) -> Option<String> {
    let generate = &project.config.generate;
    if let Some(ref specifier) = generate.schema_module_specifier {
        return Some(specifier.clone());
    }
    let path = match generate.schema_output {
        Some(ref schema_output) => path_to_ts(relative_path(
            file_name,
            &project.root_dir.join(schema_output),
        )),
        None => {
            let (schema_file, _) = schema_module_file(project, schema)?;
            relative_path(file_name, schema_file)
        }
    };
    // Module specifiers always use forward slashes.
    Some(path.to_string_lossy().replace('\\', "/"))
}

/// Converts recorded spans into mappings.
fn to_mappings(
    spans: &[RecordedSpan],
    file_index: usize,
    generated: &str,
    original: &LineIndex,
    encoding: PositionEncoding,
) -> Vec<SpanMapping> {
    spans
        .iter()
        .filter(|span| span.original_pos.file == file_index)
        .filter_map(|span| {
            let (original_start, original_len) =
                original.name_span(&span.original_pos, &span.original_name)?;
            let generated_end = span.generated_start + span.generated_len;
            let generated_start = encode_offset(generated, span.generated_start, encoding);
            let generated_len = encode_offset(generated, generated_end, encoding) - generated_start;
            let kind = if generated[span.generated_start..generated_end] == span.original_name {
                SpanMapKind::Verbatim
            } else {
                SpanMapKind::Atom
            };
            Some((
                generated_start,
                generated_len,
                original_start,
                original_len,
                kind,
            ))
        })
        .collect()
}

/// Converts errors into diagnostics of the file with given index.
/// Errors positioned in other files are reported at the start of the file.
fn to_diagnostics<'e, 'p>(
    errors: impl IntoIterator<Item = &'e PositionedError>,
    file_index: usize,
    file_path: &dyn Fn(usize) -> Option<&'p PathBuf>,
    index: &LineIndex,
    code: u32,
) -> Vec<MapperDiagnostic> {
    let describe = |pos: &Pos, message: &str| match file_path(pos.file) {
        Some(path) if !pos.builtin => format!(
            "{}:{}:{}: {message}",
            normalize_path(path).display(),
            pos.line + 1,
            pos.column + 1
        ),
        _ => message.to_owned(),
    };
    errors
        .into_iter()
        .map(|err| {
            let message = err.inner().to_string();
            let span = err
                .position()
                .filter(|pos| pos.file == file_index && !pos.builtin)
                .and_then(|pos| index.token_span(&pos));
            let mut message_text = match (span, err.position()) {
                (Some(_), _) | (None, None) => message,
                (None, Some(pos)) => describe(&pos, &message),
            };
            for (pos, info) in err.additional_info() {
                message_text.push('\n');
                message_text.push_str(&describe(pos, info));
            }
            let (start, length) = span.unwrap_or((0, 0));
            MapperDiagnostic {
                message_text,
                start,
                length,
                code,
            }
        })
        .collect()
}
