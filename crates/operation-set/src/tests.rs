use std::{
    convert::Infallible,
    path::{Path, PathBuf},
};

use nitrogql_ast::operation::ExecutableDefinition;

use super::OperationSet;

fn definition_names(document: &nitrogql_ast::OperationDocument) -> Vec<String> {
    document
        .definitions
        .iter()
        .map(|def| match def {
            ExecutableDefinition::OperationDefinition(op) => op
                .name
                .map_or("(anonymous)".to_owned(), |name| name.name.to_owned()),
            ExecutableDefinition::FragmentDefinition(fragment) => fragment.name.name.to_owned(),
        })
        .collect()
}

const MAIN: &str = r#"
#import Frag1 from "./frag1.graphql"
query Main {
    ...Frag1
}
"#;

const FRAG1: &str = r#"
#import Frag2 from "./sub/../frag2.graphql"
fragment Frag1 on Query {
    ...Frag2
}
"#;

const FRAG2: &str = "fragment Frag2 on Query { foo }";

#[test]
fn resolves_imports() {
    let mut set = OperationSet::new();
    set.add_file(Path::new("/app/main.graphql"), MAIN, 0);
    set.add_file(Path::new("/app/frag1.graphql"), FRAG1, 1);
    set.add_file(Path::new("/app/frag2.graphql"), FRAG2, 2);
    assert!(set.syntax_errors().is_empty());
    assert!(set.errors().is_empty());
    assert!(set.missing_imports().is_empty());

    let document = set
        .resolve_imports(Path::new("/app/main.graphql"))
        .unwrap()
        .unwrap();
    assert_eq!(definition_names(&document), ["Main", "Frag2", "Frag1"]);
    // each file keeps its own file index
    let ExecutableDefinition::FragmentDefinition(frag2) = &document.definitions[1] else {
        panic!("expected a fragment");
    };
    assert_eq!(frag2.position.file, 2);
}

#[test]
fn reports_missing_imports() {
    let mut set = OperationSet::new();
    set.add_file(Path::new("/app/main.graphql"), MAIN, 0);
    let missing = set.missing_imports();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].0, PathBuf::from("/app/frag1.graphql"));
    // positioned at the import path in the importing file
    assert_eq!(&MAIN[missing[0].1.span()], "\"./frag1.graphql\"");
    assert_eq!(missing[0].1.file, 0);

    set.add_file(Path::new("/app/frag1.graphql"), FRAG1, 1);
    // paths are normalized
    let missing = set.missing_imports();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].0, PathBuf::from("/app/frag2.graphql"));

    // resolving fails while imports are missing
    let err = set
        .resolve_imports(Path::new("/app/main.graphql"))
        .unwrap()
        .unwrap_err();
    assert_eq!(err.position().unwrap().file, 1);
}

#[test]
fn loads_imports() {
    let mut set = OperationSet::new();
    set.add_file(Path::new("/app/./main.graphql"), MAIN, 0);
    let mut read_files = vec![];
    set.load_imports(|path| {
        read_files.push(path.to_owned());
        let index = read_files.len();
        match path.to_str().unwrap() {
            "/app/frag1.graphql" => Ok::<_, Infallible>((FRAG1, index)),
            "/app/frag2.graphql" => Ok((FRAG2, index)),
            path => panic!("unexpected read of {path}"),
        }
    });
    assert_eq!(
        read_files,
        [
            PathBuf::from("/app/frag1.graphql"),
            PathBuf::from("/app/frag2.graphql")
        ]
    );
    assert!(set.errors().is_empty());
    // lookups normalize paths too
    let document = set
        .resolve_imports(Path::new("/app/main.graphql"))
        .unwrap()
        .unwrap();
    assert_eq!(definition_names(&document), ["Main", "Frag2", "Frag1"]);
    assert_eq!(
        set.file_path(2),
        Some(Path::new("/app/frag2.graphql")),
        "imported files are added with the index returned by `read`"
    );
}

#[test]
fn read_errors() {
    let mut set = OperationSet::new();
    set.add_file(Path::new("/app/main.graphql"), MAIN, 0);
    let mut reads = 0;
    set.load_imports(|_| {
        reads += 1;
        Err(anyhow::anyhow!("file not found"))
    });
    // a file that cannot be read is not read again
    assert_eq!(reads, 1);
    assert!(set.missing_imports().is_empty());
    assert_eq!(set.errors().len(), 1);
    let err = &set.errors()[0];
    assert_eq!(err.inner().to_string(), "file not found");
    assert_eq!(&MAIN[err.position().unwrap().span()], "\"./frag1.graphql\"");
    assert!(set.get(Path::new("/app/frag1.graphql")).is_none());
    assert!(set.contains(Path::new("/app/frag1.graphql")));
}

#[test]
fn collects_errors() {
    let mut set = OperationSet::new();
    set.add_file(Path::new("/app/syntax.graphql"), "query {", 3);
    set.add_file(
        Path::new("/app/extension.graphql"),
        "#import * from \"./a.graphql\"\n#import * from \"./a.graphql\"\nquery { foo }",
        4,
    );
    set.add_file(Path::new("/app/ok.graphql"), "query { foo }", 5);
    assert_eq!(set.syntax_errors().len(), 1);
    assert_eq!(set.syntax_errors()[0].position().unwrap().file, 3);
    assert_eq!(set.errors().len(), 1);
    assert_eq!(set.errors()[0].position().unwrap().file, 4);
    // only successfully loaded files are available
    let files: Vec<_> = set.files().map(|file| file.file_index).collect();
    assert_eq!(files, [5]);
    assert!(
        set.resolve_imports(Path::new("/app/syntax.graphql"))
            .is_none()
    );
    // failed files are not reported as missing
    assert!(set.contains(Path::new("/app/syntax.graphql")));
    assert_eq!(set.file_path(3), Some(Path::new("/app/syntax.graphql")));

    assert_eq!(set.take_syntax_errors().len(), 1);
    assert!(set.syntax_errors().is_empty());
    assert_eq!(set.errors().len(), 1);
    assert_eq!(set.take_errors().len(), 1);
    assert!(set.errors().is_empty());
}

#[test]
fn adding_twice_has_no_effect() {
    let mut set = OperationSet::new();
    set.add_file(Path::new("/app/a.graphql"), "query A { foo }", 0);
    set.add_file(Path::new("/app/./a.graphql"), "query B { foo }", 1);
    let files: Vec<_> = set.files().collect();
    assert_eq!(files.len(), 1);
    assert_eq!(definition_names(files[0].document), ["A"]);
}

#[test]
fn detects_imports_of_unloaded_files() {
    let mut set = OperationSet::new();
    set.add_file(Path::new("/app/main.graphql"), MAIN, 0);
    set.add_file(Path::new("/app/frag1.graphql"), FRAG1, 1);
    // nothing is unloaded while an import is simply missing
    assert!(!set.imports_unloaded_file(Path::new("/app/main.graphql")));

    set.add_file(Path::new("/app/frag2.graphql"), "fragment {", 2);
    // transitively, through frag1.graphql
    assert!(set.imports_unloaded_file(Path::new("/app/main.graphql")));
    assert!(set.imports_unloaded_file(Path::new("/app/frag1.graphql")));
    assert!(!set.imports_unloaded_file(Path::new("/app/frag2.graphql")));
}

#[test]
fn detects_imports_of_unloaded_files_in_cycles() {
    let mut set = OperationSet::new();
    set.add_file(
        Path::new("/app/a.graphql"),
        "#import * from \"./b.graphql\"\nfragment A on Query { foo }",
        0,
    );
    set.add_file(
        Path::new("/app/b.graphql"),
        "#import * from \"./a.graphql\"\nfragment B on Query { foo }",
        1,
    );
    assert!(!set.imports_unloaded_file(Path::new("/app/a.graphql")));
}
