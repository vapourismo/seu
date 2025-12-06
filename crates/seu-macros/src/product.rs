use syn::Expr;
use syn::Pat;
use syn::Type;
use syn::parse_quote;

/// Construct a product type from an iterator of types.
pub fn product_ty<I>(items: I) -> Type
where
    I: IntoIterator<Item = Type>,
    I::IntoIter: DoubleEndedIterator,
{
    let mut ty = parse_quote! {
        ::seu::core::product::Nil
    };

    for item in items.into_iter().rev() {
        ty = parse_quote! {
            ::seu::core::product::Cons<#item, #ty>
        };
    }

    ty
}

/// Construct a term-level product from an iterator of terms.
pub fn product_expr<I>(items: I) -> Expr
where
    I: IntoIterator<Item = Expr>,
    I::IntoIter: DoubleEndedIterator,
{
    let mut expr = parse_quote! {
        ::seu::core::product::Nil
    };

    for item in items.into_iter().rev() {
        expr = parse_quote! {
            ::seu::core::product::Cons(#item, #expr)
        };
    }

    expr
}

/// Construct a product pattern from an iterator of patterns.
pub fn product_pat<I>(items: I) -> Pat
where
    I: IntoIterator<Item = Pat>,
    I::IntoIter: DoubleEndedIterator,
{
    let mut pat = parse_quote! {
        ::seu::core::product::Nil
    };

    for item in items.into_iter().rev() {
        pat = parse_quote! {
            ::seu::core::product::Cons(#item, #pat)
        };
    }

    pat
}
