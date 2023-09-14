use std::fmt::Display;

/// Kind of input file.
#[derive(Debug, Copy, Clone)]
pub enum InputFileKind {
    Schema,
    Operation,
}

impl Display for InputFileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputFileKind::Schema => write!(f, "schema"),
            InputFileKind::Operation => write!(f, "operation"),
        }
    }
}

/// Kind of output file.
#[derive(Debug, Copy, Clone)]
pub enum OutputFileKind {
    SchemaTypeDefinition,
    SchemaTypeDefinitionSourceMap,
    ResolversTypeDefinition,
    ResolversTypeDefinitionSourceMap,
    OperationTypeDefinition,
    OperationTypeDefinitionSourceMap,
    GraphqlSource,
    GraphqlSourceSourceMap,
}

impl OutputFileKind {
    /// Convert self to corresponding source map kind.
    /// If self is already a source map kind, return self.
    pub fn to_source_map_kind(self) -> Self {
        match self {
            OutputFileKind::SchemaTypeDefinition => OutputFileKind::SchemaTypeDefinitionSourceMap,
            OutputFileKind::SchemaTypeDefinitionSourceMap => {
                OutputFileKind::SchemaTypeDefinitionSourceMap
            }
            OutputFileKind::ResolversTypeDefinition => {
                OutputFileKind::ResolversTypeDefinitionSourceMap
            }
            OutputFileKind::ResolversTypeDefinitionSourceMap => {
                OutputFileKind::ResolversTypeDefinitionSourceMap
            }
            OutputFileKind::OperationTypeDefinition => {
                OutputFileKind::OperationTypeDefinitionSourceMap
            }
            OutputFileKind::OperationTypeDefinitionSourceMap => {
                OutputFileKind::OperationTypeDefinitionSourceMap
            }
            OutputFileKind::GraphqlSource => OutputFileKind::GraphqlSourceSourceMap,
            OutputFileKind::GraphqlSourceSourceMap => OutputFileKind::GraphqlSourceSourceMap,
        }
    }
}

impl Display for OutputFileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFileKind::SchemaTypeDefinition => write!(f, "schemaTypeDefinition"),
            OutputFileKind::SchemaTypeDefinitionSourceMap => {
                write!(f, "schemaTypeDefinitionSourceMap")
            }
            OutputFileKind::ResolversTypeDefinition => write!(f, "resolversTypeDefinition"),
            OutputFileKind::ResolversTypeDefinitionSourceMap => {
                write!(f, "resolversTypeDefinitionSourceMap")
            }
            OutputFileKind::OperationTypeDefinition => write!(f, "operationTypeDefinition"),
            OutputFileKind::OperationTypeDefinitionSourceMap => {
                write!(f, "operationTypeDefinitionSourceMap")
            }
            OutputFileKind::GraphqlSource => write!(f, "graphqlSource"),
            OutputFileKind::GraphqlSourceSourceMap => write!(f, "graphqlSourceSourceMap"),
        }
    }
}
