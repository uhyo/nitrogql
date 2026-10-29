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
    OperationDocument, TypeSystemDocument,
    operation::{ExecutableDefinition, OperationDefinition},
};
use nitrogql_parser::parse_operation_document;
use nitrogql_semantics::{ast_to_type_system, resolve_operation_extensions};
use sourcemap_writer::JustWriter;

use super::{TYPE_SYSTEM, type_system, type_system_from};
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
    #[allow(dead_code, reason = "no case is known to be wider at the moment")]
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
        name: "alias_same_as_name",
        main: "query Test { me { id id: id posts { id } ...F } }",
        modules: &[(
            "f",
            "fragment F on User { name name: name posts: posts { id } }",
        )],
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
        name: "same_variable_in_operation_and_fragment",
        main: "query Test($flag: Boolean!) { me { name @include(if: $flag) ...F } }",
        modules: &[("f", "fragment F on User { age @include(if: $flag) }")],
        expect: Expect::Exact,
    },
    Case {
        name: "same_variable_in_operation_and_fragment_with_others",
        main: "query Test($a: Boolean!, $b: Boolean!, $c: Boolean!) { me { name @include(if: $a) id @skip(if: $b) ...F } }",
        modules: &[(
            "f",
            "fragment F on User { age @include(if: $a) type @include(if: $c) }",
        )],
        expect: Expect::Exact,
    },
    Case {
        name: "same_variable_in_two_fragments",
        main: "query Test($flag: Boolean!) { me { ...F ...G } }",
        modules: &[
            ("f", "fragment F on User { age @include(if: $flag) }"),
            ("g", "fragment G on User { name @skip(if: $flag) }"),
        ],
        expect: Expect::Exact,
    },
    Case {
        name: "same_variable_in_skipped_fragment",
        main: "query Test($flag: Boolean!) { me { id ...F @include(if: $flag) } }",
        modules: &[("f", "fragment F on User { age @skip(if: $flag) name }")],
        expect: Expect::Exact,
    },
    Case {
        name: "same_variable_in_nested_merging",
        main: "query Test($flag: Boolean!) { me { name @include(if: $flag) } ...F }",
        modules: &[(
            "f",
            "fragment F on Query { me { age @include(if: $flag) } }",
        )],
        expect: Expect::Exact,
    },
    Case {
        name: "same_variable_in_fragment_chain",
        main: "query Test($flag: Boolean!) { me { name @include(if: $flag) ...A } }",
        modules: &[
            ("a", "fragment A on User { id @include(if: $flag) ...B }"),
            ("b", "fragment B on User { age @skip(if: $flag) }"),
        ],
        expect: Expect::Exact,
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
    let schema = type_system();
    fs::write(out_dir.join("schema.ts"), print_schema(&schema)).unwrap();
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
    for case in CASES {
        write_case(
            &out_dir.join(case.name),
            case,
            Imports::All,
            &schema,
            "../schema",
        );
    }
    // Larger cases import only what they spread, like real projects, as costs depend on it.
    for case in [stress_case(), colocation_case(false), colocation_case(true)] {
        write_case(
            &out_dir.join(case.name),
            case,
            Imports::Spread,
            &schema,
            "../schema",
        );
    }
    let (case, schema) = wide_case();
    let dir = out_dir.join(case.name);
    write_case(&dir, case, Imports::Spread, &schema, "./schema");
    fs::write(dir.join("schema.ts"), print_schema(&schema)).unwrap();
}

/// Which modules each module of a case imports.
#[derive(Clone, Copy, Debug)]
enum Imports {
    /// All other modules, so that lookups through several wildcard imports are covered.
    All,
    /// Modules that define the fragments it spreads.
    Spread,
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
/// With `variables`, the operations and the fragments include fields by the same variable.
fn colocation_case(variables: bool) -> &'static Case {
    let operations = std::env::var("NITROGQL_FRAGMENT_REF_COLOCATION_OPERATIONS")
        .map_or(20, |value| value.parse().unwrap());
    let leak = |s: String| -> &'static str { Box::leak(s.into_boxed_str()) };
    let mut modules = vec![];
    for idx in 0..operations * 2 {
        let fields = match (idx % 2 == 0, variables) {
            (true, false) => "id title tags",
            (true, true) => "id title @skip(if: $flag) tags",
            (false, false) => "id body",
            (false, true) => "id body @include(if: $flag)",
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
                "fragment UserCard{idx} on User {{ id name{} posts {{ ...PostItem{} ...PostItem{} }} }}",
                if variables {
                    " age @include(if: $flag)"
                } else {
                    ""
                },
                idx * 2,
                idx * 2 + 1
            )),
        ));
    }
    let (variable_definitions, own_fields) = if variables {
        ("($flag: Boolean!)", " age @skip(if: $flag)")
    } else {
        ("", "")
    };
    let main = (0..operations)
        .map(|idx| {
            format!(
                "query Q{idx}{variable_definitions} {{ me {{ id{own_fields} ...UserCard{idx} ...UserCard{} }} posts {{ id ...PostItem{idx} }} }}",
                (idx + 1) % operations
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Box::leak(Box::new(Case {
        name: if variables {
            "colocation_variables"
        } else {
            "colocation"
        },
        main: leak(main),
        modules: modules.leak(),
        expect: Expect::Exact,
    }))
}

/// Case with a schema shaped like larger ones: object types have many fields, and an interface
/// has many implementations. Operations select from both, as in the colocation case.
fn wide_case() -> (&'static Case, TypeSystemDocument<'static>) {
    const TYPES: usize = 20;
    const FIELDS: usize = 30;
    let operations = std::env::var("NITROGQL_FRAGMENT_REF_COLOCATION_OPERATIONS")
        .map_or(20, |value| value.parse().unwrap());
    let leak = |s: String| -> &'static str { Box::leak(s.into_boxed_str()) };
    let fields = (0..FIELDS)
        .map(|idx| format!("f{idx}: String!"))
        .collect::<Vec<_>>()
        .join(" ");
    let types = (0..TYPES)
        .map(|idx| format!("type Item{idx} implements Node {{ id: ID! {fields} }}"))
        .collect::<Vec<_>>()
        .join("\n");
    let schema = type_system_from(leak(format!(
        "{TYPE_SYSTEM}\ninterface Node {{ id: ID! }}\n{types}\n\
         extend type Query {{ node: Node! items: [Item0!]! }}\n"
    )));
    let mut modules = vec![];
    for idx in 0..operations {
        modules.push((
            leak(format!("itemCard{idx}")),
            leak(format!(
                "fragment ItemCard{idx} on Item0 {{ id f{} f{} f{} }}",
                idx % FIELDS,
                (idx + 1) % FIELDS,
                (idx + 2) % FIELDS
            )),
        ));
        modules.push((
            leak(format!("nodeCard{idx}")),
            leak(format!(
                "fragment NodeCard{idx} on Node {{ id ... on Item{} {{ f0 f1 }} ... on Item{} {{ f2 }} }}",
                idx % TYPES,
                (idx + 1) % TYPES
            )),
        ));
    }
    let main = (0..operations)
        .map(|idx| {
            format!(
                "query Q{idx} {{ items {{ id f0 ...ItemCard{idx} }} node {{ id ...NodeCard{idx} ...NodeCard{} }} }}",
                (idx + 1) % operations
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let case = Box::leak(Box::new(Case {
        name: "wide",
        main: leak(main),
        modules: modules.leak(),
        expect: Expect::Exact,
    }));
    (case, schema)
}

fn print_schema(schema: &TypeSystemDocument) -> String {
    let mut result = String::new();
    let mut writer = JustWriter::new(&mut result);
    SchemaTypePrinter::new(SchemaTypePrinterOptions::default(), &mut writer)
        .print_document(schema)
        .unwrap();
    result
}

/// Writes modules of given case, which import schema types from `schema_source`.
fn write_case(
    dir: &Path,
    case: &Case,
    imports: Imports,
    schema: &TypeSystemDocument,
    schema_source: &str,
) {
    fs::create_dir_all(dir).unwrap();
    let main = parse(case.main, 0);
    let modules = case
        .modules
        .iter()
        .enumerate()
        .map(|(idx, (name, source))| (*name, parse(source, idx + 1)))
        .collect::<Vec<_>>();
    // Each module imports fragments of other modules, as specified by `imports`.
    // Every other module is imported with a wildcard import, so that both kinds are covered.
    // With `Imports::Spread`, the main module imports specific fragments: it has all operations
    // of the case, and fragments not imported specifically are looked up in all wildcard imports,
    // which real modules with a few imports each do not do.
    let fragment_imports = |except: Option<&str>, source: &str| {
        let wildcard = |idx: usize| match imports {
            Imports::All => idx % 2 == 1,
            Imports::Spread => except.is_some() && idx % 2 == 1,
        };
        modules
            .iter()
            .enumerate()
            .filter(|(_, (module, _))| Some(*module) != except)
            .filter(|(_, (_, doc))| match imports {
                Imports::All => true,
                Imports::Spread => fragment_names(doc).any(|name| spreads(source, name)),
            })
            .map(|(idx, (module, doc))| FragmentImport {
                module: format!("./{module}"),
                fragments: (!wildcard(idx))
                    .then(|| fragment_names(doc).map(str::to_owned).collect()),
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
    let print = |doc: &OperationDocument, fragment_imports| {
        print(schema, schema_source, doc, fragment_imports)
    };
    fs::write(dir.join("expanded.ts"), print(&expanded, None)).unwrap();
    // Fragments in other modules are referred to.
    fs::write(
        dir.join("main.ts"),
        print(&main, Some(fragment_imports(None, case.main))),
    )
    .unwrap();
    for ((module, doc), (_, source)) in modules.iter().zip(case.modules) {
        fs::write(
            dir.join(format!("{module}.ts")),
            print(doc, Some(fragment_imports(Some(module), source))),
        )
        .unwrap();
    }
    fs::write(dir.join("check.ts"), check_module(&main, case.expect)).unwrap();
}

fn parse(source: &'static str, file_index: usize) -> OperationDocument<'static> {
    let doc = parse_operation_document(source, file_index).unwrap();
    resolve_operation_extensions(doc).unwrap().0
}

/// Returns whether given source spreads a fragment of given name.
fn spreads(source: &str, name: &str) -> bool {
    source
        .match_indices(&format!("...{name}"))
        .any(|(idx, spread)| {
            !source[idx + spread.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_')
        })
}

fn fragment_names<'a>(doc: &'a OperationDocument) -> impl Iterator<Item = &'a str> {
    doc.definitions.iter().filter_map(|def| match def {
        ExecutableDefinition::FragmentDefinition(def) => Some(def.name.name),
        ExecutableDefinition::OperationDefinition(_) => None,
    })
}

/// Prints given document. Modules that import fragments from other modules are given
/// `fragment_imports`, and export their selection trees as well.
fn print(
    schema: &TypeSystemDocument,
    schema_source: &str,
    doc: &OperationDocument,
    fragment_imports: Option<Vec<FragmentImport>>,
) -> String {
    let schema = ast_to_type_system(schema);
    let options = OperationTypePrinterOptions {
        base_options: OperationBasePrinterOptions {
            export_input_type: true,
            export_result_type: true,
            ..Default::default()
        },
        schema_source: schema_source.to_owned(),
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
