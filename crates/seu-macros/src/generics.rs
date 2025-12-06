use syn::GenericArgument;
use syn::GenericParam;
use syn::parse_quote;

/// Convert a generic parameter to a generic argument.
pub fn generic_param_to_arg(param: &GenericParam) -> GenericArgument {
    match param {
        GenericParam::Lifetime(lifetime) => GenericArgument::Lifetime(lifetime.lifetime.clone()),
        GenericParam::Type(typ) => {
            let name = &typ.ident;
            GenericArgument::Type(parse_quote!(#name))
        }
        GenericParam::Const(cons) => {
            let name = &cons.ident;
            GenericArgument::Const(parse_quote!(#name))
        }
    }
}
