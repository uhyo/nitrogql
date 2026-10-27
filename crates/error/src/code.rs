use std::fmt::Display;

/// Defines `ErrorCode` along with the list of all codes.
macro_rules! error_codes {
    ($($(#[doc = $doc:literal])* $name:ident = $value:literal,)*) => {
        /// Stable code that identifies the kind of an error.
        ///
        /// Codes are part of the public interface: once published, a code must not be
        /// renumbered or reused for a different error. Codes are documented on the website
        /// (`references/error-codes`).
        ///
        /// Codes are grouped by area:
        /// - 1xxx: syntax errors
        /// - 2xxx: errors found in both schemas and operations
        /// - 3xxx: errors in schemas
        /// - 4xxx: errors in operations
        /// - 5xxx: errors in `#import`
        /// - 8xxx: errors from plugins
        /// - 9xxx: errors outside GraphQL documents, and internal errors
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        #[repr(u32)]
        pub enum ErrorCode {
            $($(#[doc = $doc])* $name = $value,)*
        }

        impl ErrorCode {
            /// All error codes.
            pub const ALL: &[ErrorCode] = &[$(ErrorCode::$name,)*];
        }
    };
}

error_codes! {
    /// Syntax error in a GraphQL document.
    SyntaxError = 1001,

    /// Type is not defined.
    UnknownType = 2001,
    /// Directive is not defined.
    UnknownDirective = 2002,
    /// Directive is not allowed for this location.
    DirectiveLocationNotAllowed = 2003,
    /// Non-repeatable directive is applied more than once.
    RepeatedDirective = 2004,
    /// Arguments are given to something that does not accept arguments.
    ArgumentsNotNeeded = 2005,
    /// Argument is not defined.
    UnknownArgument = 2006,
    /// Required argument is not specified.
    RequiredArgumentNotSpecified = 2007,
    /// Value does not match the expected type.
    TypeMismatch = 2008,
    /// Variable is not defined.
    UnknownVariable = 2009,
    /// Value is not a member of the enum.
    UnknownEnumMember = 2010,
    /// Field of an input object is not defined.
    UnknownInputField = 2011,
    /// Required field of an input object is not specified.
    RequiredInputFieldNotSpecified = 2012,
    /// Value of a @oneOf input object does not have exactly one field.
    OneOfInputNotExactlyOneField = 2013,
    /// Field of a @oneOf input object is null.
    OneOfInputNullValue = 2014,
    /// Variable used for a field of a @oneOf input object is nullable.
    OneOfInputNullableVariable = 2015,
    /// Output type is used where an input type is expected.
    OutputTypeNotAllowed = 2016,
    /// Input type is used where an output type is expected.
    InputTypeNotAllowed = 2017,

    /// Name that starts with `__` is reserved.
    ReservedName = 3001,
    /// Name is duplicated.
    DuplicatedName = 3002,
    /// Type, directive or schema is declared more than once.
    DuplicatedDeclaration = 3003,
    /// Extension has no original declaration.
    ExtensionWithoutDeclaration = 3004,
    /// Directive references itself.
    RecursingDirective = 3005,
    /// Implemented type is not an interface.
    NotInterface = 3006,
    /// Type does not implement an interface that its interfaces implement.
    InterfaceNotImplemented = 3007,
    /// Interface implements itself.
    InterfaceImplementsItself = 3008,
    /// Type does not have a field of its interface.
    InterfaceFieldNotImplemented = 3009,
    /// Type of a field does not match the field of the interface.
    FieldTypeMismatchWithInterface = 3010,
    /// Field does not have an argument of the same field of the interface.
    InterfaceArgumentNotImplemented = 3011,
    /// Type of an argument does not match the argument of the interface.
    ArgumentTypeMismatchWithInterface = 3012,
    /// Additional argument of a field from an interface is non-null.
    ArgumentTypeNonNullAgainstInterface = 3013,
    /// Member of a union is not an object type.
    NonObjectUnionMember = 3014,
    /// Field of a @oneOf input object is of non-null type.
    OneOfFieldNotNullable = 3015,
    /// Field of a @oneOf input object has a default value.
    OneOfFieldWithDefaultValue = 3016,

    /// Unnamed operation is not the only operation in the document.
    UnnamedOperationNotSingle = 4001,
    /// Operation name is duplicated.
    DuplicatedOperationName = 4002,
    /// Fragment name is duplicated.
    DuplicatedFragmentName = 4003,
    /// Root type for the operation is not defined.
    NoRootType = 4004,
    /// Fields are selected on a type that does not have fields.
    SelectionOnInvalidType = 4005,
    /// Selection set is missing for a field of an object type.
    MissingSelectionSet = 4006,
    /// Field is not found on the type.
    FieldNotFound = 4007,
    /// Variable name is duplicated.
    DuplicatedVariableName = 4008,
    /// Type is not a valid fragment target.
    InvalidFragmentTarget = 4009,
    /// Fragment is not defined.
    UnknownFragment = 4010,
    /// Fragment condition never matches its scope type.
    FragmentConditionNeverMatches = 4011,
    /// Fragment spreads itself.
    RecursingFragmentSpread = 4012,
    /// Subscription operation does not have exactly one root field.
    SubscriptionNotSingleRootField = 4013,
    /// Fields with the same response name cannot be merged.
    FieldsConflict = 4014,

    /// Wildcard import is specified more than once.
    ImportWildcardMoreThanOnce = 5001,
    /// Wildcard import is combined with specific imports.
    ImportWildcardCombinedWithSpecific = 5002,
    /// Imported file is not found.
    ImportFileNotFound = 5003,
    /// Imported fragment is not found in the imported file.
    ImportedFragmentNotFound = 5004,

    /// Error reported by a plugin.
    PluginError = 8001,

    /// Invalid content mapper option.
    InvalidMapperOption = 9001,
    /// Problem outside GraphQL documents, such as in the config file.
    ProjectError = 9002,
    /// Internal error of nitrogql.
    InternalError = 9999,
}

impl ErrorCode {
    /// Returns the numeric value of the code.
    pub fn number(self) -> u32 {
        self as u32
    }

    /// Returns the URL of the documentation for the code.
    pub fn url(self) -> String {
        format!("https://nitrogql.vercel.app/references/error-codes#{self}")
    }
}

/// Formats the code as shown to users, e.g. `nitrogql1001`.
impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "nitrogql{}", self.number())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::ErrorCode;

    #[test]
    fn codes_are_documented() {
        let page = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../website/src/app/(docs)/references/error-codes/page.tsx");
        let page = fs::read_to_string(page).unwrap();
        for code in ErrorCode::ALL {
            assert!(
                page.contains(&format!("\"{code}\"")),
                "{code} is not documented"
            );
        }
    }
}
