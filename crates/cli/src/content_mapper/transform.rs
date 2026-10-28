use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use nitrogql_ast::base::Pos;
use nitrogql_checker::{OperationCheckContext, check_operation_document};
use nitrogql_error::{ErrorCode, PositionedError};
use nitrogql_operation_set::OperationSet;
use nitrogql_printer::{
    OperationTypePrinterOptions, SchemaTypePrinter, SchemaTypePrinterOptions,
    fragment_imports::{fragment_imports_of, is_imported_fragment},
    print_fallback_types_for_operation_document, print_types_for_operation_document_with_fallback,
};
use nitrogql_utils::{normalize_path, relative_path};
use sourcemap_writer::{RecordedSpan, SpanWriter};
use thiserror::Error;

use crate::{
    generate::path_to_ts,
    schema_loader::{LoadedSchema, SchemaFileKind, schema_kind_by_path},
};

use super::{
    fallback::fallback_definitions,
    position::SourceText,
    project::{LoadedProjectSchema, Project, SchemaModule},
    protocol::{MapperDiagnostic, PositionEncoding, SpanMapKind, SpanMapping, TransformResult},
};

/// Virtual content for files that do not export anything.
const EMPTY_MODULE: &str = "export {};\n";
/// Virtual content for operation files that cannot be parsed.
/// Importing files find no fragments in it.
const EMPTY_OPERATION_MODULE: &str = "export type __NitrogqlFragments = {};\n";

#[derive(Debug, Error)]
enum TransformError {
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
    let (_, loaded_content, _) = &project.file_store()[file_index];
    // Positions are valid only if content is the same as the one loaded.
    let is_same_content = *loaded_content == content;
    let index = SourceText::new(content, encoding);
    let files = |idx: usize| {
        project
            .file_store()
            .get_file(idx)
            .map(|(path, _, _)| path.as_path())
    };
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
                result.diagnostics = to_diagnostics(errors_in_file, file_index, &files, &index);
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

    // Schema types do not change during the project's lifetime.
    let module = project.schema_module().get_or_init(|| {
        let mut options = SchemaTypePrinterOptions::from_config(&project.config);
        // Content-mapped files only provide types.
        options.emit_schema_runtime = false;
        let mut writer = SpanWriter::new();
        let mut printer = SchemaTypePrinter::new(options, &mut writer);
        printer.print_document(document)?;
        let (text, spans) = writer.into_parts();
        Ok(SchemaModule { text, spans })
    });
    match module {
        Ok(module) => {
            if is_same_content {
                result.mappings =
                    to_mappings(&module.spans, file_index, &module.text, &index, encoding);
            }
            result.text = module.text.clone();
        }
        Err(err) => {
            result.diagnostics = to_diagnostics([err], file_index, &files, &index);
        }
    }
    result
}

/// Transforms an operation file into type definitions.
///
/// Fragments imported with `#import` are not read. Instead, the module imports their selection
/// trees from the imported files, so that its output does not depend on other files.
/// Spreads of imported fragments are therefore not validated here; `nitrogql check` does it.
///
/// If the file has errors but can be parsed, a fallback module is emitted along with diagnostics.
/// It has the same exports as usual but types are `any`, so that importing files keep working.
/// Validation errors only make the affected definitions `any`;
/// other errors (e.g. in the project) affect the whole file.
fn transform_operation_file(
    project: &Project,
    file_name: &Path,
    content: &str,
    encoding: PositionEncoding,
) -> TransformResult {
    // Operation files are numbered after schema files.
    let file_index = project.file_store().schema_len();
    let mut operations = OperationSet::new();
    operations.add_file(file_name, content, file_index);
    let file_path = |idx: usize| {
        if idx < file_index {
            project
                .file_store()
                .get_file(idx)
                .map(|(path, _, _)| path.as_path())
        } else {
            operations.file_path(idx)
        }
    };
    let index = SourceText::new(content, encoding);
    let file = operations.get(file_name);
    let imports = file
        .map(|file| fragment_imports_of(file.extension, content))
        .unwrap_or_default();
    let options = || {
        let mut options = OperationTypePrinterOptions::from_config(&project.config);
        // Runtime values are provided by loaders, not by content-mapped files.
        options.print_values = false;
        options.fragment_imports = imports.clone();
        options.export_fragment_trees = true;
        options
    };

    let error_result = |diagnostics: Vec<MapperDiagnostic>| {
        let Some(file) = file else {
            return TransformResult {
                text: EMPTY_OPERATION_MODULE.to_owned(),
                extension: ".ts",
                mappings: vec![],
                diagnostics,
            };
        };
        let mut writer = SpanWriter::new();
        print_fallback_types_for_operation_document(options(), file.document, &mut writer);
        let (text, spans) = writer.into_parts();
        TransformResult {
            mappings: to_mappings(&spans, file_index, &text, &index, encoding),
            text,
            extension: ".ts",
            diagnostics,
        }
    };
    let diagnose =
        |errors: &[PositionedError]| to_diagnostics(errors, file_index, &file_path, &index);

    if !operations.syntax_errors().is_empty() {
        return error_result(diagnose(operations.syntax_errors()));
    }
    if !operations.errors().is_empty() {
        return error_result(diagnose(operations.errors()));
    }
    let Some(file) = file else {
        return error_result(vec![]);
    };
    let schema = match project.schema() {
        Ok(schema) => schema,
        Err(errors) => {
            let mut diagnostics = diagnose(errors);
            // Errors in the project are not errors in this file.
            for diagnostic in diagnostics.iter_mut() {
                diagnostic.code = ErrorCode::ProjectError.number();
                diagnostic.message_text = format!(
                    "Failed to load nitrogql project: {}",
                    diagnostic.message_text
                );
            }
            return error_result(diagnostics);
        }
    };

    let document = file.document;
    let is_imported = |name: &str| is_imported_fragment(&imports, name);
    let check_errors = check_operation_document(
        document,
        &OperationCheckContext::new(&schema.type_system).with_imported_fragments(&is_imported),
    )
    .into_iter()
    .map(PositionedError::from)
    .collect::<Vec<_>>();
    let mut diagnostics = diagnose(&check_errors);
    // Definitions with errors get `any` types, so that other definitions keep their types.
    let Some(fallback) = fallback_definitions(document, &check_errors) else {
        return error_result(diagnostics);
    };

    let Some(schema_source) = schema_source(project, schema, file_name) else {
        diagnostics.extend(diagnose(&[TransformError::NoSchemaSource.into()]));
        return error_result(diagnostics);
    };
    let mut options = options();
    options.schema_source = schema_source;

    let mut writer = SpanWriter::new();
    let printer_errors = print_types_for_operation_document_with_fallback(
        options,
        &schema.type_system,
        document,
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
    diagnostics.extend(diagnose(&printer_errors));
    let (text, spans) = writer.into_parts();
    TransformResult {
        mappings: to_mappings(&spans, file_index, &text, &index, encoding),
        text,
        extension: ".ts",
        diagnostics,
    }
}

/// Returns the schema file that provides schema types, along with its index in the file store.
/// This is used only if schema types are not provided by other means.
fn schema_module_file<'a>(
    project: &'a Project,
    schema: &LoadedProjectSchema<'_>,
) -> Option<&'a (PathBuf, usize)> {
    let generate = &project.config.generate;
    if generate.schema_output.is_some() || generate.schema_module_specifier.is_some() {
        return None;
    }
    if !matches!(schema.schema, LoadedSchema::GraphQL(_)) {
        return None;
    }
    project
        .schema_files
        .iter()
        .find(|(path, _)| matches!(schema_kind_by_path(path), SchemaFileKind::GraphQL))
}

/// Determines the module specifier to import schema types from.
fn schema_source(
    project: &Project,
    schema: &LoadedProjectSchema<'_>,
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
    original: &SourceText,
    encoding: PositionEncoding,
) -> Vec<SpanMapping> {
    let generated_text = SourceText::new(generated, encoding);
    spans
        .iter()
        .filter(|span| span.original_pos.file == file_index)
        .filter_map(|span| {
            let (original_start, original_len) =
                original.name_span(&span.original_pos, &span.original_name)?;
            let generated_end = span.generated_start + span.generated_len;
            let generated_start = generated_text.encode(span.generated_start);
            let generated_len = generated_text.encode(generated_end) - generated_start;
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
/// Errors without codes are problems outside GraphQL documents, such as in the config file.
fn to_diagnostics<'e, 'p>(
    errors: impl IntoIterator<Item = &'e PositionedError>,
    file_index: usize,
    file_path: &dyn Fn(usize) -> Option<&'p Path>,
    index: &SourceText,
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
                .and_then(|pos| index.span(&pos));
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
                code: err.code().unwrap_or(ErrorCode::ProjectError).number(),
            }
        })
        .collect()
}
