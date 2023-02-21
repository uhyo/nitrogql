use super::{utils::PairExt, Rule};
use pest::iterators::Pair;

use crate::graphql_parser::ast::{
    base::Pos,
    r#type::{ListType, NamedType, NonNullType, Type},
};

/// Builds Type from given Pair for Type.
pub fn build_type(pair: Pair<Rule>) -> Type {
    return build_type_of(pair.only_child());

    fn build_type_of(pair: Pair<Rule>) -> Type {
        match pair.as_rule() {
            Rule::NonNullType => {
                let child = pair.only_child();
                match child.as_rule() {
                    Rule::NamedType | Rule::ListType => Type::NonNull(Box::new(NonNullType {
                        r#type: build_type_of(child),
                    })),
                    rule => panic!("Unexpected rule as child of NonNullType: {:?}", rule),
                }
            }
            Rule::ListType => {
                let position: Pos = (&pair).into();
                let child = pair.only_child();
                Type::List(Box::new(ListType {
                    position,
                    r#type: build_type(child),
                }))
            }
            Rule::NamedType => Type::Named(NamedType {
                name: pair.only_child().into(),
            }),
            rule => panic!("Unexpected rule as child of Type: {:?}", rule),
        }
    }
}
