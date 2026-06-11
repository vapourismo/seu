use quote::format_ident;
use syn::DataEnum;
use syn::Expr;
use syn::FieldValue;
use syn::Fields;
use syn::Ident;
use syn::Pat;
use syn::Type;
use syn::parse_quote;

use crate::derive::variants::variant_repr;
use crate::derive::variants::variant_to_repr_arm;
use crate::names::NameGen;
use crate::product::product_expr;
use crate::product::product_pat;
use crate::product::product_ty;

/// Generate the enum type representation.
pub fn enum_repr(
    ident: &Ident,
    data: &DataEnum,
    names: &mut NameGen,
    transform_type: impl Fn(&Type) -> Type,
) -> Type {
    let variants = data
        .variants
        .iter()
        .enumerate()
        .map(|(idx, var)| variant_repr(idx, var, names, &transform_type));
    let repr = product_ty(variants);
    let variant_names = names.insert_variants(data.variants.iter()).clone();
    let name = names.insert(ident);

    parse_quote! {
        ::seu::core::enums::Enum<#name, #variant_names, #repr>
    }
}

/// Generate an expression that converts an enum to its representation.
pub fn enum_to_repr_expr(data: &DataEnum) -> Expr {
    if data.variants.is_empty() {
        return parse_quote! {
            unreachable!("Enum has no variants")
        };
    }

    let variants = data
        .variants
        .iter()
        .enumerate()
        .map(|(idx, var)| variant_to_repr_arm(idx, var));

    parse_quote! {
        ::seu::core::enums::Enum::new(
            match self {
                #(#variants),*
            }
        )
    }
}

/// Generate the expression that reconstructs the enum from its representation.
pub fn enum_from_repr_expr(data: &DataEnum) -> Expr {
    let handlers = data.variants.iter().map(|var| -> Expr {
        let names = var
            .fields
            .iter()
            .enumerate()
            .map(|(idx, _field)| format_ident!("field{idx}"));

        let var_name = &var.ident;

        let ctor: Expr = match &var.fields {
            Fields::Named(_) => {
                let ctor_bindings =
                    var.fields
                        .iter()
                        .enumerate()
                        .map(|(idx, field)| -> FieldValue {
                            let name = field
                                .ident
                                .as_ref()
                                .expect("Fields in struct variant should have a name");
                            let value_name = format_ident!("field{}", idx);
                            parse_quote! {
                                #name: #value_name
                            }
                        });
                parse_quote! {
                    Self::#var_name {
                        #(#ctor_bindings),*
                    }
                }
            }
            Fields::Unnamed(_) => {
                let ctor_names = names.clone();
                parse_quote! {
                    Self::#var_name(
                        #(#ctor_names),*
                    )
                }
            }
            Fields::Unit => parse_quote! {
                Self::#var_name
            },
        };

        let fields_pat = product_pat(names.map(|name| -> Pat {
            parse_quote! {
               ::seu::core::fields::Field { value: #name, .. }
            }
        }));

        parse_quote! {
            |var| {
                let ::seu::core::variants::Variant {
                    fields: #fields_pat,
                    ..
                } = var;
                #ctor
            }
        }
    });

    let reducer_expr = product_expr(handlers);
    parse_quote! {
        repr.variant.reduce(#reducer_expr)
    }
}
