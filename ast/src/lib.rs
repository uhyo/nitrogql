pub mod base;
mod current_file;
pub mod directive;
pub mod operation;
pub mod selection_set;
pub mod r#type;
pub mod type_system;
pub mod value;
pub mod variable;

pub use current_file::set_current_file_of_pos;
pub use operation::OperationDocument;
pub use type_system::{TypeSystemDocument, TypeSystemOrExtensionDocument};
