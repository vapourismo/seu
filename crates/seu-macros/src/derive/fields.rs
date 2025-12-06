use syn::Expr;
use syn::Field;
use syn::Fields;
use syn::Index;
use syn::Member;
use syn::Type;
use syn::parse_quote;
use syn::spanned::Spanned;

use crate::derive::names::name_marker_type;
use crate::names::NameGen;
use crate::product::product_expr;
use crate::product::product_ty;

/// Generate the representation type for a field.
///
/// `transform_type` is called to generate the representational field type, given its original type.
pub fn field_repr(
    field: &Field,
    names: &mut NameGen,
    transform_type: impl FnOnce(&Type) -> Type,
) -> Type {
    let name = name_marker_type(field, names);
    let typ = transform_type(&field.ty);

    parse_quote! {
        ::seu::core::fields::Field<#name, #typ>
    }
}

/// Generate the representation type for a collection of fields.
///
/// `transform_type` is called to generate the representational field type, given its original type.
pub fn fields_repr(
    fields: &Fields,
    names: &mut NameGen,
    transform_type: impl Fn(&Type) -> Type,
) -> Type {
    let field_reprs = fields
        .iter()
        .map(|field| field_repr(field, names, &transform_type));
    product_ty(field_reprs)
}

/// Generate an expression that instantiates a field's representation.
pub fn field_to_repr_expr(
    idx: usize,
    field: &Field,
    transform_value: impl FnOnce(Expr) -> Expr,
) -> Expr {
    let member = match &field.ident {
        Some(ident) => Member::Named(ident.clone()),
        None => Member::Unnamed(Index {
            index: idx as u32,
            span: field.span(),
        }),
    };

    let value = transform_value(parse_quote! { self.#member });

    parse_quote! {
        ::seu::core::fields::Field::new(#value)
    }
}

/// Generate an expression that instantiates the product for all field representations.
pub fn fields_to_repr_expr(fields: &Fields, transform_value: impl Fn(Expr) -> Expr) -> Expr {
    let fields = fields
        .iter()
        .enumerate()
        .map(|(idx, field)| field_to_repr_expr(idx, field, &transform_value));
    product_expr(fields)
}
