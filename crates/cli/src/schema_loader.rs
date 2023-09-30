use std::{borrow::Cow, io, path::Path};

use graphql_type_system::Schema;
use nitrogql_ast::base::Pos;
use nitrogql_config_file::run_node;

pub enum SchemaFileKind {
    GraphQL,
    IntrospectionJson,
    SchemaJavaScript,
}

pub fn schema_kind_by_path(path: &Path) -> SchemaFileKind {
    let ext = path.extension().and_then(|ext| ext.to_str());
    match ext {
        Some("graphql") => SchemaFileKind::GraphQL,
        Some("json") => SchemaFileKind::IntrospectionJson,
        Some("js" | "mjs" | "cjs" | "ts" | "mts" | "cts") => SchemaFileKind::SchemaJavaScript,
        _ => SchemaFileKind::GraphQL,
    }
}

pub fn load_schema_js(path: &Path) -> io::Result<String> {
    run_node(&format!(
        r#"
import {{ loadSchemaJs }} from "@nitrogql/core";
import {{ stdout }} from "process";

loadSchemaJs("{}").then((schema) => {{
    stdout.write(schema);
}});
"#,
        path.display()
    ))
}

#[allow(clippy::large_enum_variant)]
pub enum LoadedSchema<'src, Gql> {
    GraphQL(Gql),
    Introspection(Schema<Cow<'src, str>, Pos>),
}

impl<'src, Gql> LoadedSchema<'src, Gql> {
    pub fn map_into<'a, F, G, R>(&'a self, graphql: F, introspection: G) -> R
    where
        F: FnOnce(&'a Gql) -> R,
        G: FnOnce(&'a Schema<Cow<'src, str>, Pos>) -> R,
    {
        match self {
            LoadedSchema::GraphQL(gql) => graphql(gql),
            LoadedSchema::Introspection(schema) => introspection(schema),
        }
    }
}
