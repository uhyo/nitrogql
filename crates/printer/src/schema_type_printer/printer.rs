use std::collections::HashMap;

use nitrogql_ast::type_system::{TypeDefinition, TypeSystemDefinition, TypeSystemDocument};
use nitrogql_config_file::{Config, ScalarTypeConfig, TypeTarget};
use nitrogql_semantics::ast_to_type_system;
use sourcemap_writer::SourceMapWriter;

use crate::{schema::get_builtin_scalar_types, ts_types::TSType};

use super::{
    context::SchemaTypePrinterContext, error::SchemaTypePrinterResult, type_printer::TypePrinter,
};

pub struct SchemaTypePrinterOptions {
    /// Type of each scalar. Provided as raw TypeScript code.
    pub scalar_types: HashMap<String, ScalarTypeConfig>,
    /// Special type name for types that includes schema metadata
    pub schema_metadata_type: String,
    /// Whether to make input nullable fields optional.
    pub input_nullable_field_is_optional: bool,
    /// Whether to emit runtime for generated schema types.
    pub emit_schema_runtime: bool,
    /// Whether to emit utility types for merging fragments defined in other modules.
    pub emit_fragment_merge_utilities: bool,
}

impl Default for SchemaTypePrinterOptions {
    fn default() -> Self {
        SchemaTypePrinterOptions {
            scalar_types: get_builtin_scalar_types(),
            schema_metadata_type: "__nitrogql_schema".into(),
            input_nullable_field_is_optional: true,
            emit_schema_runtime: false,
            emit_fragment_merge_utilities: false,
        }
    }
}

impl SchemaTypePrinterOptions {
    /// Generate from config.
    pub fn from_config(config: &Config) -> Self {
        let mut result = SchemaTypePrinterOptions {
            emit_schema_runtime: config.generate.emit_schema_runtime,
            input_nullable_field_is_optional: config
                .generate
                .r#type
                .allow_undefined_as_optional_input,
            ..SchemaTypePrinterOptions::default()
        };
        result.scalar_types.extend(
            config
                .generate
                .r#type
                .scalar_types
                .iter()
                .map(|(key, value)| (key.to_owned(), value.clone())),
        );
        result
    }
}

pub struct SchemaTypePrinter<'a, Writer: SourceMapWriter> {
    options: SchemaTypePrinterOptions,
    writer: &'a mut Writer,
}

impl<'a, Writer> SchemaTypePrinter<'a, Writer>
where
    Writer: SourceMapWriter,
{
    pub fn new(options: SchemaTypePrinterOptions, writer: &'a mut Writer) -> Self {
        SchemaTypePrinter { options, writer }
    }

    pub fn print_document(&mut self, document: &TypeSystemDocument) -> SchemaTypePrinterResult<()> {
        let schema = ast_to_type_system(document);
        self.print_prelude(document);

        for target in [
            TypeTarget::OperationInput,
            TypeTarget::OperationOutput,
            TypeTarget::ResolverInput,
            TypeTarget::ResolverOutput,
        ] {
            writeln!(self.writer, "export declare namespace {target} {{");
            self.writer.indent();
            let context = SchemaTypePrinterContext::new(&self.options, document, &schema, target);
            for def in document.definitions.iter() {
                def.print_type(&context, self.writer)?;
                self.writer.write("\n");
            }
            self.writer.dedent();
            writeln!(self.writer, "}}\n");
        }

        let context = SchemaTypePrinterContext::new(
            &self.options,
            document,
            &schema,
            // target is dummy
            TypeTarget::OperationOutput,
        );
        for def in document.definitions.iter() {
            def.print_representative(&context, self.writer)?;
            self.writer.write("\n");
        }

        Ok(())
    }

    fn print_prelude(&mut self, document: &TypeSystemDocument) {
        self.writer.write("export type ");
        self.writer.write(&self.options.schema_metadata_type);
        self.writer.write(" = ");
        let schema_metadata_type = get_schema_metadata_type(document);
        schema_metadata_type.print_type(self.writer);
        self.writer.write(";\n\n");
        // Print utility types
        self.writer.write(
            "type __Beautify<Obj> = { [K in keyof Obj]: Obj[K] } & {};
export type __SelectionSet<Orig, Obj, Others> =
  __Beautify<Pick<{
    [K in keyof Orig]: Obj extends { [P in K]?: infer V } ? V : unknown
  }, Extract<keyof Orig, keyof Obj>> & Others>;

",
        );
        if self.options.emit_fragment_merge_utilities {
            self.writer.write(FRAGMENT_MERGE_UTILITIES);
        }
    }
}
/// Utility types for merging selections of fragments defined in other modules.
///
/// Such selections are printed unrendered: each concrete object type in a selection is a
/// `__Branch` with its fields. Fields of fragments are merged into branches of the same object
/// type, and `__Render` converts the result into the same types as fully expanded selections.
const FRAGMENT_MERGE_UTILITIES: &str = r#"export type __Branch<Name extends string, Orig, Fields, Aliased> = {
  " $name": Name;
  " $orig": Orig;
  " $fields": Fields;
  " $aliased": Aliased;
};
type __AnyBranch = __Branch<string, unknown, unknown, unknown>;
type __IsAny<T> = 0 extends 1 & T ? true : false;
// Fields omitted by `@skip` or `@include` are optional `never`, which read as `undefined` (or `never`).
// Mapping over keys of both keeps a field optional only if it is omitted on both sides.
// Values are erased before intersecting, as conflicting values would reduce it to `never`.
type __Keys<T> = { [K in keyof T]: unknown };
type __MergeFields<A, B> = [keyof A & keyof B] extends [never] ? A & B : __MergeOverlappingFields<A, B>;
type __MergeOverlappingFields<A, B> = {
  [K in keyof (__Keys<A> & __Keys<B>)]: K extends keyof A
    ? K extends keyof B ? __MergeValue<A[K], B[K]> : A[K]
    : K extends keyof B ? B[K] : never;
};
type __MergeValue<A, B> =
  [A] extends [undefined] ? B
  : [B] extends [undefined] ? A
  : __MergeMember<A, B>;
type __MergeMember<A, B> =
  A extends __AnyBranch ? __MergeBranch<A, B>
  : A extends readonly (infer E)[] ? __MergeValue<E, B extends readonly (infer F)[] ? F : never>[]
  : A;
type __MergeBranch<A extends __AnyBranch, B, Matching = Extract<B, { " $name": A[" $name"] }>> =
  [Matching] extends [never] ? A : __MergeMatchingBranches<A, Matching>;
// Distributes over branches of the same object type, which differ in variable values.
type __MergeMatchingBranches<A extends __AnyBranch, B> =
  B extends __AnyBranch
    ? __Branch<A[" $name"], A[" $orig"], __MergeFields<A[" $fields"], B[" $fields"]>, __MergeFields<A[" $aliased"], B[" $aliased"]>>
    : never;
export type __MergeFragment<A, Fragment> =
  __IsAny<Fragment> extends true ? any
  : A extends __AnyBranch ? __MergeBranch<A, Fragment> : A;
// Unlike merging into a selection, branches of object types that only one fragment has are kept.
export type __CombineFragments<A, B> =
  __IsAny<A> extends true ? any
  : __IsAny<B> extends true ? any
  : __MergeMember<A, B> | Exclude<B, { " $name": A extends __AnyBranch ? A[" $name"] : never }>;
export type __SkipFragment<Fragment> =
  Fragment extends __AnyBranch
    ? __Branch<Fragment[" $name"], Fragment[" $orig"], { [K in keyof Fragment[" $fields"]]?: never }, { [K in keyof Fragment[" $aliased"]]?: never }>
    : Fragment;
// Homomorphic, so that omitted fields stay optional.
type __RenderFields<Fields> = { [K in keyof Fields]: __Render<Fields[K]> };
// `any` needs no special handling, as it absorbs the union of both branches.
export type __Render<T> =
  T extends __AnyBranch ? __SelectionSet<T[" $orig"], __RenderFields<T[" $fields"]>, __RenderFields<T[" $aliased"]>>
  : T extends readonly (infer E)[] ? __Render<E>[]
  : T;

"#;

fn get_schema_metadata_type(document: &TypeSystemDocument) -> TSType {
    let schema_definition = document.definitions.iter().find_map(|def| match def {
        TypeSystemDefinition::SchemaDefinition(def) => Some(def),
        _ => None,
    });
    if let Some(schema_def) = schema_definition {
        return TSType::object(schema_def.definitions.iter().map(|(op, ty)| {
            (
                op.as_str(),
                TSType::TypeVariable(ty.into()),
                schema_def.description.as_ref().map(|d| d.value.clone()),
            )
        }));
    }
    // If there is no schema definition, use default root type names.
    let mut operations = vec![];
    for d in document.definitions.iter() {
        let TypeSystemDefinition::TypeDefinition(def) = d else {
            continue;
        };
        let TypeDefinition::Object(def) = def else {
            continue;
        };

        match def.name.name {
            "Query" => {
                operations.push(("query", (&def.name).into()));
            }
            "Mutation" => {
                operations.push(("mutation", (&def.name).into()));
            }
            "Subscription" => {
                operations.push(("subscription", (&def.name).into()));
            }
            _ => {}
        }
    }

    TSType::object(
        operations
            .into_iter()
            .map(|(op, ty)| (op, TSType::TypeVariable(ty), None)),
    )
}
