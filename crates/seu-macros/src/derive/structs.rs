use quote::format_ident;
use quote::quote;
use syn::DataStruct;
use syn::Expr;
use syn::Fields;
use syn::Type;
use syn::parse_quote;

use crate::derive::fields::fields_repr;
use crate::derive::fields::fields_to_repr_expr;
use crate::derive::variants::variant_flavour;
use crate::names::NameGen;
use crate::product::product_pat;

/// Generate the struct representation.
pub fn struct_repr(
    data: &DataStruct,
    names: &mut NameGen,
    transform_fields: impl Fn(&Type) -> Type,
) -> Type {
    let flavour = variant_flavour(&data.fields, names);
    let fields = fields_repr(&data.fields, names, transform_fields);

    parse_quote! {
        ::seu::core::structs::Struct<#flavour, #fields>
    }
}

/// Generate an expression that instantiates the whole struct representation.
pub fn struct_to_repr_expr(data: &DataStruct, transform_value: impl Fn(Expr) -> Expr) -> Expr {
    let fields_impl = fields_to_repr_expr(&data.fields, transform_value);
    parse_quote! {
        ::seu::core::structs::Struct::new(#fields_impl)
    }
}

/// Generate an expression that reconstructs the struct from its representation.
pub fn struct_from_repr_expr(data: &DataStruct) -> Expr {
    let field_bindings = data.fields.iter().enumerate().map(|(idx, _field)| {
        let field_name = format_ident!("field{idx}");
        parse_quote! {
            ::seu::core::fields::Field {
                value: #field_name,
                ..
            }
        }
    });

    let ctor = match &data.fields {
        Fields::Named(fields) => {
            let field_bindings = fields.named.iter().enumerate().map(|(idx, field)| {
                let name = &field.ident;
                let value_name = format_ident!("field{idx}");
                quote! {
                    #name: #value_name
                }
            });
            quote! {
                Self {
                    #(#field_bindings),*
                }
            }
        }
        Fields::Unnamed(fields) => {
            let field_names = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, _)| format_ident!("field{}", idx));
            quote! {
                Self(#(#field_names),*)
            }
        }
        Fields::Unit => quote! {
            Self
        },
    };

    let fields_pat = product_pat(field_bindings);

    parse_quote! {
        {
            let ::seu::core::structs::Struct {
                fields: #fields_pat,
                ..
            } = repr;
            #ctor
        }
    }
}
