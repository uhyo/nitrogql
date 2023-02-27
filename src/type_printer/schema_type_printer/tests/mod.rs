#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::{
        extension_resolver::resolve_extensions,
        graphql_parser::{ast::TypeSystemDocument, parser::parse_type_system_document},
        source_map_writer::just_writer::JustWriter,
        type_printer::schema_type_printer::{
            error::SchemaTypePrinterResult,
            printer::{SchemaTypePrinter, SchemaTypePrinterOptions},
        },
    };

    #[test]
    fn type_printing() {
        let doc = parse_type_system_document(
            "
            type User implements HasID {
                id: ID!
                name: String!
                type: UserType!
                age: Int
                posts: [Post!]!
            }
            interface HasID {
                id: ID!
            }
            enum UserType {
                NormalUser
                PremiumUser
            }
            type Bot implements HasID {
                id: ID!
            }

            type Post {
                id: ID!
                title: String!
                tags: [String!]
                body: String!
            }

            input UserSearchQuery {
                age: Int
                name: String
            }

            type Query {
                me: User!
            }
            ",
        )
        .unwrap();
        let doc = resolve_extensions(doc).unwrap();
        let printed = print_document(&doc).unwrap();
        assert_snapshot!(printed);
    }

    fn print_document(document: &TypeSystemDocument) -> SchemaTypePrinterResult<String> {
        let mut result = String::new();
        let mut writer = JustWriter::new(&mut result);
        let mut printer = SchemaTypePrinter::new(SchemaTypePrinterOptions::default(), &mut writer);
        printer.print_document(document)?;
        Ok(result)
    }
}
