use quote::format_ident;
use syn::Arm;
use syn::Expr;
use syn::Fields;
use syn::Pat;
use syn::Type;
use syn::Variant;
use syn::parse_quote;

use crate::derive::fields::fields_repr;
use crate::names::NameGen;
use crate::nums::num_expr;
use crate::nums::num_ty;
use crate::product::product_expr;

/// Returns the variant flavour marker type corresponding to the given fields flavour.
pub fn variant_flavour(fields: &Fields, names: &mut NameGen) -> Type {
    match fields {
        Fields::Named(_) => {
            let names = names.insert_fields(fields);
            parse_quote! {
                ::seu::core::variants::StructVariant<#names>
            }
        }
        Fields::Unnamed(_) => parse_quote! {
            ::seu::core::variants::TupleVariant
        },
        Fields::Unit => parse_quote! {
            ::seu::core::variants::UnitVariant
        },
    }
}

/// Construct the variant representation type.
pub fn variant_repr(
    idx: usize,
    var: &Variant,
    names: &mut NameGen,
    transform_type: impl Fn(&Type) -> Type,
) -> Type {
    let idx = num_ty(idx);
    let flavour = variant_flavour(&var.fields, names);
    let fields = fields_repr(&var.fields, names, transform_type);
    let name = names.insert(&var.ident);

    parse_quote! {
        ::seu::core::variants::Variant<
            #name,
            #idx,
            #flavour,
            #fields,
        >
    }
}

/// Generate the match arm for the given variant. Its body evaluates to the corresponding variant
/// representation.
pub fn variant_to_repr_arm(idx: usize, var: &Variant) -> Arm {
    let idx = num_expr(idx);
    let name = &var.ident;

    let pat: Pat = match &var.fields {
        Fields::Named(fields) => {
            let fields = fields.named.iter().map(|field| &field.ident);
            parse_quote! {
                Self::#name {
                    #(#fields),*
                }
            }
        }
        Fields::Unnamed(fields) => {
            let fields = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, _field)| format_ident!("_{}", idx));
            parse_quote! {
                Self::#name(
                    #(#fields),*
                )
            }
        }
        Fields::Unit => parse_quote! {
            Self::#name
        },
    };

    let body: Expr = {
        let names = var.fields.iter().enumerate().map(|(idx, field)| {
            let name = field
                .ident
                .clone()
                .unwrap_or_else(|| format_ident!("_{}", idx));
            parse_quote! {
                ::seu::core::fields::Field::new(
                    #name
                )
            }
        });

        product_expr(names)
    };

    parse_quote! {
        #pat => {
            ::seu::core::sum::Sum::new(
                #idx,
                ::seu::core::variants::Variant::new(#body),
            )
        }
    }
}
