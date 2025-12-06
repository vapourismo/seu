use syn::Error;
use syn::Expr;
use syn::LitInt;
use syn::Type;
use syn::parse_quote;

/// Construct a Peano number expression.
pub fn num_expr(num: usize) -> Expr {
    let mut expr = parse_quote! {
        ::seu::core::nums::Zero
    };

    for _ in 0..num {
        expr = parse_quote! {
            ::seu::core::nums::Succ(#expr)
        };
    }

    expr
}

/// Given a proc-macro input literal integer, convert it to a Peano number expression.
pub fn main_expr(input: LitInt) -> Result<Expr, Error> {
    input.base10_parse().map(num_expr)
}

/// Construct a Peano number type.
pub fn num_ty(num: usize) -> Type {
    let mut ty = parse_quote! {
        ::seu::core::nums::Zero
    };

    for _ in 0..num {
        ty = parse_quote! {
            ::seu::core::nums::Succ<#ty>
        };
    }

    ty
}

/// Given a proc-macro input literal integer, convert it to a Peano number type.
pub fn main_ty(input: LitInt) -> Result<Type, Error> {
    input.base10_parse().map(num_ty)
}
