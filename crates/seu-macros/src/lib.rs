mod derive;
mod generics;
mod names;
mod nums;
mod opts;
mod product;

/// Generate a DataType implementation for the annotated type.
///
/// This macro implements the `DataType` trait for the specified type.
#[proc_macro_derive(DataType, attributes(datatype))]
pub fn data_type_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    let output = derive::main(input).unwrap_or_else(syn::Error::into_compile_error);
    proc_macro::TokenStream::from(output)
}

/// Turn the given integer into Peano number expression.
#[proc_macro]
pub fn num(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    let output = nums::main_expr(input).map_or_else(
        syn::Error::into_compile_error,
        quote::ToTokens::into_token_stream,
    );
    proc_macro::TokenStream::from(output)
}

/// Turn the given integer into a Peano number type.
#[expect(nonstandard_style, reason = "Need to differentiate name")]
#[proc_macro]
pub fn Num(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    let output = nums::main_ty(input).map_or_else(
        syn::Error::into_compile_error,
        quote::ToTokens::into_token_stream,
    );
    proc_macro::TokenStream::from(output)
}
