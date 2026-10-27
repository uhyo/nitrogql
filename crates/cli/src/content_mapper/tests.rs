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
        None,
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
fn fields_conflict() {
    // Fields of the same response name that cannot be merged are reported by the checker
    // before the type printer tries to merge them.
    let mut files = default_files();
    let operation = "query GetMe {\n  a: __typename\n  a: me {\n    id\n  }\n}\n";
    files[5] = ("app/operation.graphql", operation);
    let root = setup("fields_conflict", &files);
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
        None,
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
    let index = utf8_result
        .mappings
        .iter()
        .position(|mapping| mapping.2 == original_start)
        .unwrap();
    // 😀 is 4 bytes in UTF-8 and 2 code units in UTF-16.
    assert_eq!(result.mappings[index].2, original_start - 2);
    // generated code is ASCII only
    assert_eq!(result.mappings[index].0, utf8_result.mappings[index].0);
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

/// Watched files of given project, relative to root.
fn watched_files(project: &Project, root: &Path) -> Vec<String> {
    project
        .watched_files
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

#[test]
fn watched_files_and_identity() {
    let root = setup("watched_files_and_identity", &default_files());
    let project = open(&root);
    assert_eq!(
        watched_files(&project, &root),
        vec![
            "graphql.config.yaml",
            "schema/a.graphql",
            "schema/b.graphql"
        ]
    );
    let identity = project.config_identity.clone();
    assert_eq!(open(&root).config_identity, identity);

    // Operation files do not affect other files, as imported fragments are not expanded.
    fs::write(
        root.join("app/fragment.graphql"),
        "fragment UserName on User { id }",
    )
    .unwrap();
    fs::write(
        root.join("app/operation.graphql"),
        format!("#import * from \"./other.graphql\"\n{OPERATION}"),
    )
    .unwrap();
    assert_eq!(open(&root).config_identity, identity);
    // Schema files do.
    fs::write(
        root.join("schema/b.graphql"),
        USER_SCHEMA.replace("age: Int", "age: Int!"),
    )
    .unwrap();
    assert_ne!(open(&root).config_identity, identity);
}

#[test]
fn wildcard_import() {
    let mut files = default_files();
    let operation = r#"#import UserName from "./fragment.graphql"
#import * from "other.graphql"
query GetMe {
  me {
    ...UserName
    ...UserAge
  }
}
"#;
    files[5] = ("app/operation.graphql", operation);
    let root = setup("wildcard_import", &files);
    let project = open(&root);
    // Paths without `./` are relative to the file as well.
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert_snapshot!(print_result(&result, operation, &root));
}

#[test]
fn transitive_import() {
    // Fragments imported by an imported file are not imported to the importing file.
    let mut files = default_files();
    files[4] = (
        "app/fragment.graphql",
        "#import UserAge from \"./age.graphql\"\nfragment UserName on User {\n  name\n  ...UserAge\n}\n",
    );
    files.push(("app/age.graphql", "fragment UserAge on User {\n  age\n}\n"));
    let operation = OPERATION.replace("...UserName", "...UserName\n    ...UserAge");
    files[5] = ("app/operation.graphql", &operation);
    let root = setup("transitive_import", &files);
    let project = open(&root);
    let result = transform_file(&project, &root, "app/fragment.graphql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let result = transform_file(&project, &root, "app/operation.graphql");
    let messages = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message_text.as_str())
        .collect::<Vec<_>>();
    assert_eq!(messages, vec!["Fragment 'UserAge' is not defined"]);
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
    // Errors of an imported file are reported in that file only.
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let result = transform_file(&project, &root, "app/fragment.graphql");
    assert_snapshot!(print_result(&result, "fragment UserName on User {", &root));
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
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    // The fragment falls back to `any` in the file that defines it,
    // which makes selections it is merged into `any` in importing files.
    let result = transform_file(&project, &root, "app/fragment.graphql");
    assert_snapshot!(print_result(
        &result,
        "fragment UserName on User {\n  nam\n}\n",
        &root
    ));
}

fn reopen(root: &Path, previous: Project) -> Project {
    block_on(Project::open(
        &root.join("tsconfig.json"),
        &MapperOptions::default(),
        Some(previous),
    ))
}

/// Whether the schema module is already printed,
/// which tells that the schema was reused from the previous project.
fn has_schema_module(project: &Project) -> bool {
    project.schema_module().get().is_some()
}

#[test]
fn reopen_unchanged() {
    let root = setup("reopen_unchanged", &default_files());
    let project = open(&root);
    let identity = project.config_identity.clone();
    let watched = project.watched_files.clone();
    let schema_module = transform_file(&project, &root, "schema/a.graphql");

    let project = reopen(&root, project);
    // The schema is reused.
    assert!(has_schema_module(&project));
    assert_eq!(project.config_identity, identity);
    assert_eq!(project.watched_files, watched);
    assert_eq!(
        transform_file(&project, &root, "schema/a.graphql"),
        schema_module
    );
    let result = transform_file(&project, &root, "app/operation.graphql");
    assert_snapshot!("operation_file", print_result(&result, OPERATION, &root));
}

#[test]
fn reopen_schema_changed() {
    let root = setup("reopen_schema_changed", &default_files());
    let project = open(&root);
    let identity = project.config_identity.clone();
    transform_file(&project, &root, "schema/a.graphql");
    fs::write(
        root.join("schema/b.graphql"),
        USER_SCHEMA.replace("age: Int", "age: Int!"),
    )
    .unwrap();
    let project = reopen(&root, project);
    assert!(!has_schema_module(&project));
    assert_ne!(project.config_identity, identity);
    let result = transform_file(&project, &root, "schema/a.graphql");
    assert!(result.text.contains("age: Int;"), "{}", result.text);
    assert!(!result.text.contains("age: Int | null"), "{}", result.text);
}

#[test]
fn reopen_config_changed() {
    let root = setup("reopen_config_changed", &default_files());
    let project = open(&root);
    let result = transform_file(&project, &root, "schema/a.graphql");
    assert!(result.text.contains("ID = string"), "{}", result.text);
    let config = format!(
        "{CONFIG}extensions:\n  nitrogql:\n    generate:\n      type:\n        scalarTypes:\n          ID: number\n"
    );
    fs::write(root.join("graphql.config.yaml"), config).unwrap();
    let project = reopen(&root, project);
    // Config may affect the schema, so it is loaded again.
    assert!(!has_schema_module(&project));
    let result = transform_file(&project, &root, "schema/a.graphql");
    assert!(result.text.contains("ID = number"), "{}", result.text);
}

/// Measures the time spent by each request on a large project.
/// Run with `cargo test --release -p nitrogql-cli content_mapper::tests::benchmark -- --ignored --nocapture`.
#[test]
#[ignore]
fn benchmark() {
    use std::time::{Duration, Instant};

    const TYPES: usize = 2000;
    const FRAGMENTS: usize = 200;
    const OPERATIONS: usize = 500;
    const IMPORTS: usize = 5;

    let mut schema = String::from("type Query {\n");
    for i in 0..TYPES {
        writeln!(schema, "  t{i}(id: ID!): T{i}").unwrap();
    }
    schema.push_str("}\ninterface Node {\n  id: ID!\n}\n");
    for i in 0..TYPES {
        writeln!(
            schema,
            "\"\"\"Type {i}\"\"\"\ntype T{i} implements Node {{\n  id: ID!\n  name: String!\n  count: Int\n  next: T{}\n  list(first: Int = 10): [T{}!]!",
            (i + 1) % TYPES,
            (i + 7) % TYPES
        )
        .unwrap();
        for f in 0..10 {
            writeln!(schema, "  field{f}: String").unwrap();
        }
        schema.push_str("}\n");
    }
    let mut files = vec![
        ("tsconfig.json".to_owned(), "{}".to_owned()),
        ("graphql.config.yaml".to_owned(), CONFIG.to_owned()),
        ("schema/schema.graphql".to_owned(), schema),
    ];
    for j in 0..FRAGMENTS {
        files.push((
            format!("app/fragments/f{j}.graphql"),
            format!(
                "fragment F{j} on T{j} {{\n  id\n  name\n  next {{\n    id\n    count\n  }}\n  list {{\n    field0\n    field1\n  }}\n}}\n"
            ),
        ));
    }
    for k in 0..OPERATIONS {
        let mut operation = String::new();
        let imported = (0..IMPORTS)
            .map(|n| (k * IMPORTS + n) % FRAGMENTS)
            .collect::<Vec<_>>();
        for j in &imported {
            writeln!(operation, "#import F{j} from \"../fragments/f{j}.graphql\"").unwrap();
        }
        writeln!(operation, "query Q{k}($id: ID!) {{").unwrap();
        for j in &imported {
            writeln!(
                operation,
                "  t{j}(id: $id) {{\n    ...F{j}\n    field2\n  }}"
            )
            .unwrap();
        }
        operation.push_str("}\n");
        files.push((format!("app/ops/q{k}.graphql"), operation));
    }
    let files = files
        .iter()
        .map(|(path, content)| (path.as_str(), content.as_str()))
        .collect::<Vec<_>>();
    let root = setup("benchmark", &files);

    fn measure<T>(name: &str, mut f: impl FnMut() -> T) -> T {
        let mut times = vec![];
        let mut result = None;
        for _ in 0..10 {
            let start = Instant::now();
            result = Some(f());
            times.push(start.elapsed());
        }
        times.sort();
        let total: Duration = times.iter().sum();
        println!(
            "{name}: median {:?}, min {:?}, mean {:?}",
            times[times.len() / 2],
            times[0],
            total / times.len() as u32
        );
        result.unwrap()
    }

    let mut project = Some(measure("open project", || open(&root)));
    // TypeScript closes and reopens the project when a watched file changes.
    measure("reopen project", || {
        project = Some(block_on(Project::open(
            &root.join("tsconfig.json"),
            &MapperOptions::default(),
            project.take(),
        )));
    });
    let project = project.unwrap();

    let operation_path = root.join("app/ops/q0.graphql");
    let operation = fs::read_to_string(&operation_path).unwrap();
    let mut n = 0;
    let result = measure("transform operation file", || {
        // TypeScript requests a transform when the content changes.
        n += 1;
        let content = format!("{operation}# {n}\n");
        transform(&project, &operation_path, &content, PositionEncoding::Utf8)
    });
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);

    let schema_path = root.join("schema/schema.graphql");
    let schema = fs::read_to_string(&schema_path).unwrap();
    for encoding in [PositionEncoding::Utf8, PositionEncoding::Utf16] {
        let start = Instant::now();
        let project = open(&root);
        let opened = start.elapsed();
        let result = transform(&project, &schema_path, &schema, encoding);
        println!(
            "transform schema file ({encoding:?}, first): {:?}",
            start.elapsed() - opened
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        measure(&format!("transform schema file ({encoding:?})"), || {
            transform(&project, &schema_path, &schema, encoding)
        });
    }
    let _ = fs::remove_dir_all(&root);
}
