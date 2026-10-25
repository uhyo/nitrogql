use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use futures::executor::block_on;
use insta::assert_snapshot;

use super::{
    project::{MapperOptions, Project},
    protocol::{PositionEncoding, TransformResult},
    transform::transform,
};

/// Creates a project directory with given files.
fn setup(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "nitrogql-content-mapper-test-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    for (path, content) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
    root
}

fn open(root: &Path) -> Project {
    block_on(Project::open(
        &root.join("tsconfig.json"),
        &MapperOptions::default(),
    ))
}

fn transform_file(project: &Project, root: &Path, path: &str) -> TransformResult {
    let path = root.join(path);
    let content = fs::read_to_string(&path).unwrap();
    transform(project, &path, &content, PositionEncoding::Utf8)
}

/// Prints transform result with mapped spans annotated.
fn print_result(result: &TransformResult, original: &str, root: &Path) -> String {
    let mut output = result.text.clone();
    for (virtual_start, virtual_len, original_start, original_len, kind) in &result.mappings {
        writeln!(
            output,
            "// mapping: {:?} -> {:?} ({kind:?})",
            &result.text[*virtual_start..virtual_start + virtual_len],
            &original[*original_start..original_start + original_len],
        )
        .unwrap();
    }
    for diagnostic in &result.diagnostics {
        writeln!(
            output,
            "// diagnostic {}: {:?} at {:?}",
            diagnostic.code,
            diagnostic.message_text,
            &original[diagnostic.start..diagnostic.start + diagnostic.length],
        )
        .unwrap();
    }
    output.replace(&root.to_string_lossy().into_owned(), "<root>")
}

const CONFIG: &str = "
schema: ./schema/*.graphql
documents: ./app/**/*.graphql
";

const SCHEMA: &str = "type Query {
  me: User!
}
";

const USER_SCHEMA: &str = "type User {
  id: ID!
  name: String!
  age: Int
}
";

const FRAGMENT: &str = "fragment UserName on User {
  name
}
";

const OPERATION: &str = r#"#import UserName from "./fragment.graphql"
# 😀
query GetMe {
  me {
    id
    ...UserName
  }
}
"#;

fn default_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("tsconfig.json", "{}"),
        ("graphql.config.yaml", CONFIG),
        ("schema/a.graphql", SCHEMA),
        ("schema/b.graphql", USER_SCHEMA),
        ("app/fragment.graphql", FRAGMENT),
        ("app/operation.graphql", OPERATION),
    ]
}

#[test]
fn operation_file() {
    let root = setup("operation_file", &default_files());
    let project = open(&root);
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert_snapshot!(print_result(&result, OPERATION, &root));
}

#[test]
fn fragment_file() {
    let root = setup("fragment_file", &default_files());
    let project = open(&root);
    let result = transform_file(&project, &root, "app/fragment.graphql");
    assert_snapshot!(print_result(&result, FRAGMENT, &root));
}

#[test]
fn schema_files() {
    let root = setup("schema_files", &default_files());
    let project = open(&root);
    // The first schema file provides schema types.
    let result = transform_file(&project, &root, "schema/a.graphql");
    assert!(
        result
            .text
            .contains("export declare namespace __OperationOutput")
    );
    assert!(
        result
            .text
            .contains("export type User = __OperationOutput.User;")
    );
    assert!(result.diagnostics.is_empty());
    // Other schema files are empty modules.
    let result = transform_file(&project, &root, "schema/b.graphql");
    assert_eq!(result.text, "export {};\n");
    assert!(result.mappings.is_empty());
}

#[test]
fn schema_types_from_schema_output() {
    let mut files = default_files();
    let config = format!(
        "{CONFIG}extensions:\n  nitrogql:\n    generate:\n      schemaOutput: ./generated/schema.d.ts\n"
    );
    files[1] = ("graphql.config.yaml", &config);
    let root = setup("schema_types_from_schema_output", &files);
    let project = open(&root);
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert!(
        result
            .text
            .contains(r#"import type * as Schema from "../generated/schema.js";"#),
        "{}",
        result.text
    );
    // Schema files are not used for providing types.
    let result = transform_file(&project, &root, "schema/a.graphql");
    assert_eq!(result.text, "export {};\n");
}

#[test]
fn validation_error() {
    let mut files = default_files();
    let operation = "query GetMe {\n  me {\n    nam\n  }\n}\n";
    files[5] = ("app/operation.graphql", operation);
    let root = setup("validation_error", &files);
    let project = open(&root);
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert_snapshot!(print_result(&result, operation, &root));
}

#[test]
fn diagnostic_spans() {
    // Diagnostics cover the node they point to, cut at the end of its first line.
    let mut files = default_files();
    let operation = "query GetMe {\n  me {\n    id @unknown\n    ...Missing\n    ... on Query { me { id } }\n    age {\n      x\n    }\n  }\n}\n";
    files[5] = ("app/operation.graphql", operation);
    let root = setup("diagnostic_spans", &files);
    let project = open(&root);
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert_snapshot!(print_result(&result, operation, &root));
}

#[test]
fn printer_error() {
    // The checker does not check that fields of the same response name can be merged,
    // but the type printer cannot print such a selection.
    let mut files = default_files();
    let operation = "query GetMe {\n  a: __typename\n  a: me {\n    id\n  }\n}\n";
    files[5] = ("app/operation.graphql", operation);
    let root = setup("printer_error", &files);
    let project = open(&root);
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert_snapshot!(print_result(&result, operation, &root));
}

#[test]
fn syntax_error() {
    let mut files = default_files();
    let operation = "query GetMe {\n  me {\n";
    files[5] = ("app/operation.graphql", operation);
    let root = setup("syntax_error", &files);
    let project = open(&root);
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert_snapshot!(print_result(&result, operation, &root));
}

#[test]
fn schema_error() {
    let mut files = default_files();
    let schema = "type Query {\n  me: Usr!\n}\n";
    files[2] = ("schema/a.graphql", schema);
    let root = setup("schema_error", &files);
    let project = open(&root);
    // Errors are reported in the schema file
    let result = transform_file(&project, &root, "schema/a.graphql");
    assert_snapshot!(print_result(&result, schema, &root));
    // and operation files
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert_snapshot!(print_result(&result, OPERATION, &root));
}

#[test]
fn config_not_found() {
    let mut files = default_files();
    files.remove(1);
    let root = setup("config_not_found", &files);
    let project = open(&root);
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert_snapshot!(print_result(&result, OPERATION, &root));
}

#[test]
fn config_file_option() {
    let mut files = default_files();
    files[1] = (
        "config/nitrogql.yaml",
        "schema: ../schema/*.graphql\ndocuments: ../app/**/*.graphql\n",
    );
    let root = setup("config_file_option", &files);
    let project = block_on(Project::open(
        &root.join("tsconfig.json"),
        &MapperOptions {
            config_file: Some("./config/nitrogql.yaml".into()),
        },
    ));
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(project.root_dir, root.join("config"));
}

#[test]
fn utf16_positions() {
    let root = setup("utf16_positions", &default_files());
    let project = open(&root);
    let path = root.join("app/operation.graphql");
    let result = transform(&project, &path, OPERATION, PositionEncoding::Utf16);
    let utf8_result = transform(&project, &path, OPERATION, PositionEncoding::Utf8);
    let original_start = OPERATION.find("GetMe").unwrap();
    // 😀 is 4 bytes in UTF-8 and 2 code units in UTF-16.
    assert_eq!(utf8_result.mappings[0].2, original_start);
    assert_eq!(result.mappings[0].2, original_start - 2);
    // generated code is ASCII only
    assert_eq!(result.mappings[0].0, utf8_result.mappings[0].0);
}

#[test]
fn unsaved_content() {
    let root = setup("unsaved_content", &default_files());
    let project = open(&root);
    // Content given by TypeScript takes precedence over the file on disk.
    let content = "query GetMyName {\n  me {\n    name\n  }\n}\n";
    let result = transform(
        &project,
        &root.join("app/operation.graphql"),
        content,
        PositionEncoding::Utf8,
    );
    assert!(result.text.contains("GetMyNameQuery"), "{}", result.text);
}

#[test]
fn watched_files_and_identity() {
    let root = setup("watched_files_and_identity", &default_files());
    let project = open(&root);
    let watched = project
        .watched_files
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    // Operation files are watched only if they are imported from other files.
    assert_eq!(
        watched,
        vec![
            "app/fragment.graphql",
            "graphql.config.yaml",
            "schema/a.graphql",
            "schema/b.graphql",
        ]
    );
    let identity = project.config_identity.clone();
    assert_eq!(open(&root).config_identity, identity);

    // Changes to imported files change identity
    fs::write(
        root.join("app/fragment.graphql"),
        "fragment UserName on User { id }",
    )
    .unwrap();
    let identity2 = open(&root).config_identity.clone();
    assert_ne!(identity2, identity);
    // but changes to other operation files do not.
    fs::write(
        root.join("app/operation.graphql"),
        OPERATION.replace("    id\n", "    id\n    age\n"),
    )
    .unwrap();
    assert_eq!(open(&root).config_identity, identity2);
}

#[test]
fn imported_file_error() {
    let mut files = default_files();
    let operation = r#"#import UserName from "./fragment.graphql"
query GetMe {
  me {
    ...UserName
    ...UserAge
  }
}
fragment UserAge on User {
  age
}
"#;
    files[4] = ("app/fragment.graphql", "fragment UserName on User {");
    files[5] = ("app/operation.graphql", operation);
    let root = setup("imported_file_error", &files);
    let project = open(&root);
    // Exports of the transformed file are kept even if an imported file has errors.
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert_snapshot!(print_result(&result, operation, &root));
}

#[test]
fn validation_error_per_definition() {
    // Only definitions with errors, and those that depend on them, fall back to `any`.
    let mut files = default_files();
    let operation = r#"#import UserName from "./fragment.graphql"
query GetMe {
  me {
    ...UserName
  }
}
query GetAge {
  me {
    ...UserAge
  }
}
query GetId {
  me {
    idd
  }
}
fragment UserAge on User {
  agee
}
"#;
    files[5] = ("app/operation.graphql", operation);
    let root = setup("validation_error_per_definition", &files);
    let project = open(&root);
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert_snapshot!(print_result(&result, operation, &root));
}

#[test]
fn imported_fragment_validation_error() {
    let mut files = default_files();
    let operation = r#"#import UserName from "./fragment.graphql"
query GetMe {
  me {
    ...UserName
  }
}
query GetId {
  me {
    id
  }
}
"#;
    files[4] = (
        "app/fragment.graphql",
        "fragment UserName on User {\n  nam\n}\n",
    );
    files[5] = ("app/operation.graphql", operation);
    let root = setup("imported_fragment_validation_error", &files);
    let project = open(&root);
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert_snapshot!(print_result(&result, operation, &root));
}
