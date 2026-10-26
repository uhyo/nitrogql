//! Harness for checking types of imported fragments, which are merged at the type level.
//!
//! Each case is printed twice: once with all fragments expanded into one module
//! (as `nitrogql generate` does), and once with fragments in separate modules that import
//! each other's selection trees (as the content mapper does).
//! The generated `check.ts` of each case asserts that both give the same types.
//!
//! Run `e2e-tests/fragment-types/run.sh` to generate the cases and type-check them.

use std::{fmt::Write as _, fs, path::Path};

use nitrogql_ast::{
    OperationDocument,
    operation::{ExecutableDefinition, OperationDefinition},
};
use nitrogql_parser::parse_operation_document;
use nitrogql_semantics::{ast_to_type_system, resolve_operation_extensions};
use sourcemap_writer::JustWriter;

use super::type_system;
use crate::{
    OperationTypePrinterOptions, SchemaTypePrinter, SchemaTypePrinterOptions,
    fragment_imports::FragmentImport, operation_base_printer::options::OperationBasePrinterOptions,
    print_types_for_operation_document,
};

/// Expected relation between the expanded types and the types with fragment references.
#[derive(Clone, Copy, Debug)]
enum Expect {
    /// Both are identical.
    Exact,
    /// Types with fragment references are strictly wider than expanded types.
    Wider,
}

struct Case {
    name: &'static str,
    /// Main module. Its operations and fragments are compared.
    main: &'static str,
    /// Other modules, by name. Fragments defined here are referred to from other modules.
    modules: &'static [(&'static str, &'static str)],
    expect: Expect,
}

const CASES: &[Case] = &[
    Case {
        name: "import_fragment",
        main: "query MyQuery { me { id ...UserProfile } }",
        modules: &[("userProfile", "fragment UserProfile on User { name age }")],
        expect: Expect::Exact,
    },
    Case {
        name: "spread_interface_fragment_in_list",
        main: "query Test { me { id name posts { id ...F } } }",
        modules: &[("f", "fragment F on HasID { id }")],
        expect: Expect::Exact,
    },
    Case {
        name: "spread_conditions_on_interface",
        main: "query Test { me { posts { id ...F ...P } } }",
        modules: &[
            ("f", "fragment F on HasID { id }"),
            ("p", "fragment P on Post { title body }"),
        ],
        expect: Expect::Exact,
    },
    Case {
        name: "object_fragment_on_interface_parent",
        main: "query Test { me { posts { ...U } } }",
        modules: &[("u", "fragment U on Post { id title }")],
        expect: Expect::Exact,
    },
    Case {
        name: "interface_fragment_on_object_parent",
        main: "query Test { me { ...H name } }",
        modules: &[("h", "fragment H on HasID { id }")],
        expect: Expect::Exact,
    },
    Case {
        name: "fragment_merging",
        main: "query Test { me { ...F ...P } }",
        modules: &[
            ("f", "fragment F on User { id }"),
            ("p", "fragment P on User { name }"),
        ],
        expect: Expect::Exact,
    },
    Case {
        name: "fragment_nested_merging",
        main: "query Test { ...F ...P }",
        modules: &[(
            "fragments",
            "fragment F on Query { me { id } }\nfragment P on Query { me { name } }",
        )],
        expect: Expect::Exact,
    },
    Case {
        name: "fragment_nested_merging_list",
        main: "query Test { ...F ...P }",
        modules: &[
            ("f", "fragment F on Query { posts { id tags } }"),
            ("p", "fragment P on Query { posts { id title } }"),
        ],
        expect: Expect::Exact,
    },
    Case {
        name: "nested_merging_with_own_fields",
        main: "query Test { me { id posts { id } } ...F }",
        modules: &[(
            "f",
            "fragment F on Query { me { name posts { ... on Post { title } } } }",
        )],
        expect: Expect::Exact,
    },
    Case {
        name: "nested_merging_nullable_list",
        main: "query Test { nullablePosts { id } ...F }",
        modules: &[("f", "fragment F on Query { nullablePosts { title } }")],
        expect: Expect::Exact,
    },
    Case {
        name: "conditional_fragments",
        main: "query Test { ...F ...P }",
        modules: &[
            (
                "f",
                "fragment F on Query { me { posts { id ... on Post { title } } } }",
            ),
            (
                "p",
                "fragment P on Query { me { posts { id ... on Tweet { body } } } }",
            ),
        ],
        expect: Expect::Exact,
    },
    Case {
        name: "typename_in_fragment",
        main: "query Test { me { posts { ...T ... on Post { title } } } }",
        modules: &[("t", "fragment T on HasID { __typename id }")],
        expect: Expect::Exact,
    },
    Case {
        name: "aliases",
        main: "query Test { me { userName: name myPosts: posts { id } ...F } }",
        modules: &[(
            "f",
            "fragment F on User { userName: name myPosts: posts { ... on Post { title } } }",
        )],
        expect: Expect::Exact,
    },
    Case {
        name: "fragment_chain",
        main: "query Test { me { ...A } }",
        modules: &[
            ("a", "fragment A on User { id ...B }"),
            ("b", "fragment B on User { name posts { ...C } }"),
            ("c", "fragment C on Post { title }"),
        ],
        expect: Expect::Exact,
    },
    Case {
        name: "local_fragment_spreading_external",
        main: "query Test { me { ...Local } }\nfragment Local on User { id ...Ext }",
        modules: &[("ext", "fragment Ext on User { name }")],
        expect: Expect::Exact,
    },
    Case {
        name: "spread_in_inline_fragment",
        main: "query Test { me { posts { id ... on Post { ...P } } } }",
        modules: &[("p", "fragment P on Post { title }")],
        expect: Expect::Exact,
    },
    Case {
        name: "skip_fragment_spread",
        main: "query Test { me { id name type ...F @skip(if: true) } }",
        modules: &[("f", "fragment F on User { age }")],
        expect: Expect::Exact,
    },
    Case {
        name: "include_fragment_spread",
        main: "query Test { me { id name type ...F @include(if: true) } }",
        modules: &[("f", "fragment F on User { age }")],
        expect: Expect::Exact,
    },
    Case {
        name: "skip_and_include_fragment_spread",
        main: "query Test { me { id name type ...F @skip(if: true) @include(if: true) } }",
        modules: &[("f", "fragment F on User { age }")],
        expect: Expect::Exact,
    },
    Case {
        name: "skip_fragment_spread_overlapping_own_field",
        main: "query Test { me { id age ...F @skip(if: true) } }",
        modules: &[("f", "fragment F on User { age name }")],
        expect: Expect::Exact,
    },
    Case {
        name: "skip_fragment_spread_with_aliases",
        main: "query Test($skip: Boolean!) { me { n: name ...F @skip(if: $skip) } }",
        modules: &[("f", "fragment F on User { a: age n: name p: posts { id } }")],
        expect: Expect::Exact,
    },
    Case {
        name: "skip_fragment_spread_variable",
        main: "query Test($skip: Boolean!) { me { id name type ...F @skip(if: $skip) } }",
        modules: &[("f", "fragment F on User { age }")],
        expect: Expect::Exact,
    },
    Case {
        name: "include_fragment_spread_variable",
        main: "query Test($include: Boolean!) { me { id name type ...F @include(if: $include) } }",
        modules: &[("f", "fragment F on User { age }")],
        expect: Expect::Exact,
    },
    Case {
        name: "skip_and_include_fragment_spread_variable",
        main: "query Test($flag: Boolean!) { me { id name ...F @skip(if: $flag) ... @include(if: $flag) { age } } }",
        modules: &[("f", "fragment F on User { type }")],
        expect: Expect::Exact,
    },
    Case {
        name: "spread_in_skipped_inline_fragment",
        main: "query Test($flag: Boolean!) { me { id ... @include(if: $flag) { ...F } } }",
        modules: &[("f", "fragment F on User { name }")],
        expect: Expect::Exact,
    },
    Case {
        name: "variable_inside_fragment",
        main: "query Test($withAge: Boolean!) { me { id ...F } }",
        modules: &[("f", "fragment F on User { age @include(if: $withAge) }")],
        expect: Expect::Exact,
    },
    Case {
        name: "variable_inside_two_fragments",
        main: "query Test($a: Boolean!, $b: Boolean!) { me { ...F ...G } }",
        modules: &[
            ("f", "fragment F on User { age @include(if: $a) }"),
            ("g", "fragment G on User { name @include(if: $b) }"),
        ],
        expect: Expect::Exact,
    },
    Case {
        // Branches of the fragment and of the operation are combined as if the variable
        // could have different values in each.
        name: "same_variable_in_operation_and_fragment",
        main: "query Test($flag: Boolean!) { me { name @include(if: $flag) ...F } }",
        modules: &[("f", "fragment F on User { age @include(if: $flag) }")],
        expect: Expect::Wider,
    },
    Case {
        name: "variables_in_nested_merging",
        main: "query Test($x: Boolean!, $y: Boolean!) { me { name @include(if: $x) } ...F }",
        modules: &[("f", "fragment F on Query { me { age @include(if: $y) } }")],
        expect: Expect::Exact,
    },
];

#[test]
#[ignore = "run via e2e-tests/fragment-types/run.sh"]
fn write_fragment_ref_cases() {
    let out_dir = std::env::var("NITROGQL_FRAGMENT_TYPES_OUT")
        .expect("NITROGQL_FRAGMENT_TYPES_OUT must be set");
    let out_dir = Path::new(&out_dir);
    let _ = fs::remove_dir_all(out_dir);
    fs::create_dir_all(out_dir).unwrap();
    fs::write(out_dir.join("schema.ts"), print_schema()).unwrap();
    fs::write(
        out_dir.join("fragment-types.d.ts"),
        include_str!("../../../../../packages/cli/fragment-types.d.ts"),
    )
    .unwrap();
    fs::write(
        out_dir.join("typed-document-node.ts"),
        "export interface TypedDocumentNode<Result, Variables> {\n  __result?: Result;\n  __variables?: Variables;\n}\n",
    )
    .unwrap();
    for case in CASES.iter().chain([stress_case(), colocation_case()]) {
        write_case(&out_dir.join(case.name), case);
    }
}

/// Larger case for comparing type checking costs.
/// Many fragments in separate modules are merged into the same nested selections.
fn stress_case() -> &'static Case {
    const FRAGMENTS: usize = 30;
    let leak = |s: String| -> &'static str { Box::leak(s.into_boxed_str()) };
    let main = (0..FRAGMENTS)
        .map(|idx| format!("...F{idx}"))
        .collect::<Vec<_>>()
        .join(" ");
    let modules = (0..FRAGMENTS)
        .map(|idx| {
            (
                leak(format!("f{idx}")),
                leak(format!(
                    "fragment F{idx} on Query {{ me {{ n{idx}: name posts {{ id ... on Post {{ t{idx}: title tags }} ... on Tweet {{ b{idx}: body }} }} }} nullablePosts {{ id b{idx}: body }} }}"
                )),
            )
        })
        .collect::<Vec<_>>();
    Box::leak(Box::new(Case {
        name: "stress",
        main: leak(format!("query Test {{ {main} }}")),
        modules: modules.leak(),
        expect: Expect::Exact,
    }))
}

/// Case that resembles fragment colocation: operations spread fragments of components,
/// which spread fragments of their child components.
fn colocation_case() -> &'static Case {
    let operations = std::env::var("NITROGQL_FRAGMENT_REF_COLOCATION_OPERATIONS")
        .map_or(20, |value| value.parse().unwrap());
    let leak = |s: String| -> &'static str { Box::leak(s.into_boxed_str()) };
    let mut modules = vec![];
    for idx in 0..operations * 2 {
        let fields = if idx % 2 == 0 {
            "id title tags"
        } else {
            "id body"
        };
        modules.push((
            leak(format!("postItem{idx}")),
            leak(format!("fragment PostItem{idx} on Post {{ {fields} }}")),
        ));
    }
    for idx in 0..operations {
        modules.push((
            leak(format!("userCard{idx}")),
            leak(format!(
                "fragment UserCard{idx} on User {{ id name posts {{ ...PostItem{} ...PostItem{} }} }}",
                idx * 2,
                idx * 2 + 1
            )),
        ));
    }
    let main = (0..operations)
        .map(|idx| {
            format!(
                "query Q{idx} {{ me {{ id ...UserCard{idx} ...UserCard{} }} posts {{ id ...PostItem{idx} }} }}",
                (idx + 1) % operations
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Box::leak(Box::new(Case {
        name: "colocation",
        main: leak(main),
        modules: modules.leak(),
        expect: Expect::Exact,
    }))
}

fn print_schema() -> String {
    let schema = type_system();
    let mut result = String::new();
    let mut writer = JustWriter::new(&mut result);
    SchemaTypePrinter::new(SchemaTypePrinterOptions::default(), &mut writer)
        .print_document(&schema)
        .unwrap();
    result
}

fn write_case(dir: &Path, case: &Case) {
    fs::create_dir_all(dir).unwrap();
    let main = parse(case.main, 0);
    let modules = case
        .modules
        .iter()
        .enumerate()
        .map(|(idx, (name, source))| (*name, parse(source, idx + 1)))
        .collect::<Vec<_>>();
    // Each module imports fragments of all other modules.
    // Every other module is imported with a wildcard import, so that both kinds are covered.
    let fragment_imports = |except: Option<&str>| {
        modules
            .iter()
            .enumerate()
            .filter(|(_, (module, _))| Some(*module) != except)
            .map(|(idx, (module, doc))| FragmentImport {
                module: format!("./{module}"),
                fragments: (idx % 2 == 0).then(|| fragment_names(doc).map(str::to_owned).collect()),
                source: None,
            })
            .collect::<Vec<_>>()
    };

    // All fragments expanded into one module, like `#import` does today.
    let expanded = OperationDocument {
        definitions: main
            .definitions
            .iter()
            .chain(modules.iter().flat_map(|(_, doc)| doc.definitions.iter()))
            .cloned()
            .collect(),
        position: main.position,
    };
    fs::write(dir.join("expanded.ts"), print(&expanded, None)).unwrap();
    // Fragments in other modules are referred to.
    fs::write(
        dir.join("main.ts"),
        print(&main, Some(fragment_imports(None))),
    )
    .unwrap();
    for (module, doc) in modules.iter() {
        fs::write(
            dir.join(format!("{module}.ts")),
            print(doc, Some(fragment_imports(Some(module)))),
        )
        .unwrap();
    }
    fs::write(dir.join("check.ts"), check_module(&main, case.expect)).unwrap();
}

fn parse(source: &'static str, file_index: usize) -> OperationDocument<'static> {
    let doc = parse_operation_document(source, file_index).unwrap();
    resolve_operation_extensions(doc).unwrap().0
}

fn fragment_names<'a>(doc: &'a OperationDocument) -> impl Iterator<Item = &'a str> {
    doc.definitions.iter().filter_map(|def| match def {
        ExecutableDefinition::FragmentDefinition(def) => Some(def.name.name),
        ExecutableDefinition::OperationDefinition(_) => None,
    })
}

/// Prints given document. Modules that import fragments from other modules are given
/// `fragment_imports`, and export their selection trees as well.
fn print(doc: &OperationDocument, fragment_imports: Option<Vec<FragmentImport>>) -> String {
    let schema = type_system();
    let schema = ast_to_type_system(&schema);
    let options = OperationTypePrinterOptions {
        base_options: OperationBasePrinterOptions {
            export_input_type: true,
            export_result_type: true,
            ..Default::default()
        },
        schema_source: "../schema".to_owned(),
        typed_document_node_source: "../typed-document-node".to_owned(),
        export_fragment_trees: fragment_imports.is_some(),
        fragment_imports: fragment_imports.unwrap_or_default(),
        fragment_utilities_source: "../fragment-types".to_owned(),
        ..Default::default()
    };
    let mut result = String::new();
    let mut writer = JustWriter::new(&mut result);
    print_types_for_operation_document(options, &schema, doc, &mut writer).unwrap();
    result
}

/// Module that asserts relations between expanded types and types with fragment references.
fn check_module(main: &OperationDocument, expect: Expect) -> String {
    let mut result = String::from(
        "import type * as Expanded from \"./expanded\";\nimport type * as Referred from \"./main\";\n\n\
         type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;\n\
         type Expect<T extends true> = T;\n\n",
    );
    let type_names = main.definitions.iter().map(|def| match def {
        ExecutableDefinition::OperationDefinition(OperationDefinition {
            name: Some(name), ..
        }) => format!("{}Result", name.name),
        ExecutableDefinition::OperationDefinition(_) => {
            panic!("operations in cases must be named")
        }
        ExecutableDefinition::FragmentDefinition(def) => def.name.name.to_owned(),
    });
    for (idx, type_name) in type_names.enumerate() {
        let (expanded, referred) = (
            format!("Expanded.{type_name}"),
            format!("Referred.{type_name}"),
        );
        match expect {
            Expect::Exact => writeln!(
                result,
                "export type _{idx} = Expect<Equal<{expanded}, {referred}>>;"
            ),
            Expect::Wider => writeln!(
                result,
                "export type _{idx} = [\n  Expect<Equal<Equal<{expanded}, {referred}>, false>>,\n  \
                 Expect<[{expanded}] extends [{referred}] ? true : false>,\n  \
                 Expect<[{referred}] extends [{expanded}] ? false : true>,\n];"
            ),
        }
        .unwrap();
    }
    result
}
