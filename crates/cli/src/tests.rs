use std::{
    fs,
    path::{Path, PathBuf},
};

use futures::executor::block_on;
use insta::assert_snapshot;
use typed_arena::Arena;

use crate::{Args, context::OutputFormat, file_store::FileStore, output::CliOutput};

use super::{report_result, run_cli_impl};

/// Creates a project directory with given files.
fn setup(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("nitrogql-cli-test-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    for (path, content) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
    root
}

/// Runs CLI with given commands in given project.
/// Returns the exit code and the output in given format.
fn run(root: &Path, commands: &[&str], output_format: OutputFormat) -> (i32, String) {
    let args = Args {
        config_file: Some(root.join("graphql.config.yaml")),
        schema: vec![],
        operation: vec![],
        schema_output: None,
        output_format,
        commands: commands.iter().map(|c| c.to_string()).collect(),
    };
    let sources = Arena::new();
    let mut file_store = FileStore::new(&sources);
    let mut output = CliOutput::new();
    let res = block_on(run_cli_impl(args, &mut file_store, &mut output));
    let code = report_result(res, &file_store, &mut output);
    let printed = match output_format {
        OutputFormat::Json => output.json_string(&file_store),
        OutputFormat::Rdjson => output.rdjson_string(&file_store),
        OutputFormat::Human => unimplemented!(),
    };
    // Paths are escaped in JSON output.
    let root = root.to_string_lossy().replace('/', "\\/");
    (code, printed.replace(&root, "<root>"))
}

const CONFIG: &str = "
schema: ./schema/*.graphql
documents: ./app/**/*.graphql
";

const BROKEN_SCHEMA: &str = "type Query {
  me: User
}
type User {
  id: ID! !
}
";

const SCHEMA: &str = "type Query {
  me: String
}
";

const BROKEN_OPERATION: &str = "query Me {
  me { ) }
}
";

#[test]
fn syntax_errors_json() {
    let root = setup(
        "syntax-errors-json",
        &[
            ("graphql.config.yaml", CONFIG),
            ("schema/schema.graphql", BROKEN_SCHEMA),
            ("app/me.graphql", BROKEN_OPERATION),
        ],
    );
    let (code, output) = run(&root, &["check"], OutputFormat::Json);
    assert_eq!(code, 1);
    assert_snapshot!(output);
}

#[test]
fn syntax_errors_rdjson() {
    let root = setup(
        "syntax-errors-rdjson",
        &[
            ("graphql.config.yaml", CONFIG),
            ("schema/schema.graphql", BROKEN_SCHEMA),
            ("app/me.graphql", BROKEN_OPERATION),
        ],
    );
    let (code, output) = run(&root, &["check"], OutputFormat::Rdjson);
    assert_eq!(code, 1);
    assert_snapshot!(output);
}

#[test]
fn syntax_errors_json_without_check() {
    let root = setup(
        "syntax-errors-json-without-check",
        &[
            ("graphql.config.yaml", CONFIG),
            ("schema/schema.graphql", SCHEMA),
            ("app/me.graphql", BROKEN_OPERATION),
        ],
    );
    let (code, output) = run(&root, &["generate"], OutputFormat::Json);
    assert_eq!(code, 1);
    assert_snapshot!(output);
}

#[test]
fn import_errors_do_not_hide_other_errors() {
    let root = setup(
        "import-errors-do-not-hide-other-errors",
        &[
            ("graphql.config.yaml", CONFIG),
            ("schema/schema.graphql", SCHEMA),
            // imports a file that does not exist
            (
                "app/frag.graphql",
                "#import Moved from \"./moved.graphql\"
fragment Frag on Query {
  ...Moved
}
",
            ),
            // depends on the missing file through frag.graphql
            (
                "app/user.graphql",
                "#import Frag from \"./frag.graphql\"
query User {
  ...Frag
}
",
            ),
            // has invalid imports, so it cannot be loaded
            (
                "app/invalid.graphql",
                "#import * from \"./frag.graphql\"
#import * from \"./frag.graphql\"
fragment Invalid on Query {
  me
}
",
            ),
            // the error is reported in invalid.graphql, not here
            (
                "app/uses-invalid.graphql",
                "#import Invalid from \"./invalid.graphql\"
query UsesInvalid {
  ...Invalid
}
",
            ),
            // unrelated to the broken imports
            (
                "app/other.graphql",
                "query Other {
  unknown
}
",
            ),
        ],
    );
    let (code, output) = run(&root, &["check"], OutputFormat::Json);
    assert_eq!(code, 1);
    assert_snapshot!(output);
}
