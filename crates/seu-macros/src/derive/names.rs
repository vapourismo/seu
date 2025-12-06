use syn::Field;
use syn::Type;
use syn::parse_quote;

use crate::names::NameGen;

/// Pick the marker type for a field's name.
pub fn name_marker_type(field: &Field, names: &mut NameGen) -> Type {
    match &field.ident {
        Some(ident) => {
            let name = names.insert(ident);
            parse_quote! {
                ::seu::core::names::Named<#name>
            }
        }

        None => parse_quote! {
            ::seu::core::names::Unnamed
        },
    }
}
