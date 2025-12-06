use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::Expr;
use syn::ItemImpl;
use syn::LitStr;
use syn::Type;
use syn::parse_quote;

use crate::derive::enums::enum_from_repr_expr;
use crate::derive::enums::enum_repr;
use crate::derive::enums::enum_to_repr_expr;
use crate::derive::structs::struct_from_repr_expr;
use crate::derive::structs::struct_repr;
use crate::derive::structs::struct_to_repr_expr;
use crate::generics::generic_param_to_arg;
use crate::names::NameGen;

/// Generate a type representation for a data type.
pub fn data_type_repr(
    data: &Data,
    names: &mut NameGen,
    transform_fields: impl Fn(&Type) -> Type,
) -> Result<Type, Error> {
    Ok(match data {
        Data::Struct(data_struct) => struct_repr(data_struct, names, transform_fields),
        Data::Enum(data_enum) => enum_repr(data_enum, names, transform_fields),
        Data::Union(data_union) => {
            return Err(Error::new(
                data_union.union_token.span,
                "DataType: Union data types are not yet supported",
            ));
        }
    })
}

/// Generate an expression that converts a data type to its representation.
pub fn data_type_to_repr_expr(
    data: &Data,
    transform_value: impl Fn(Expr) -> Expr,
) -> Result<Expr, Error> {
    Ok(match data {
        Data::Struct(data_struct) => struct_to_repr_expr(data_struct, transform_value),
        Data::Enum(data_enum) => enum_to_repr_expr(data_enum),
        Data::Union(data_union) => {
            return Err(Error::new(
                data_union.union_token.span,
                "DataType: Union data types are not yet supported",
            ));
        }
    })
}

/// Generate an expression that constructs the data type from its representation.
pub fn data_type_from_repr_expr(data: &Data) -> Result<Expr, Error> {
    Ok(match data {
        Data::Struct(data_struct) => struct_from_repr_expr(data_struct),
        Data::Enum(data_enum) => enum_from_repr_expr(data_enum),
        Data::Union(data_union) => {
            return Err(Error::new(
                data_union.union_token.span,
                "DataType: Union data types are not yet supported",
            ));
        }
    })
}

/// Generate an implementation of the DataType trait for the given data type.
pub fn data_type_impl(item: DeriveInput, names: &mut NameGen) -> Result<ItemImpl, Error> {
    let name = &item.ident;
    let ty_params = &item.generics.params;
    let where_clause = &item.generics.where_clause;

    let name_lit = LitStr::new(name.to_string().as_str(), name.span());

    let repr = data_type_repr(&item.data, names, |t| t.clone())?;
    let repr_ref = data_type_repr(&item.data, names, |t| parse_quote! { &'a #t })?;
    let repr_mut = data_type_repr(&item.data, names, |t| parse_quote! { &'a mut #t })?;

    let to_impl = data_type_to_repr_expr(&item.data, |e| e)?;
    let to_impl_ref = data_type_to_repr_expr(&item.data, |e| parse_quote! { &#e })?;
    let to_impl_mut = data_type_to_repr_expr(&item.data, |e| parse_quote! { &mut #e })?;

    let from_impl = data_type_from_repr_expr(&item.data)?;

    let ty_args = ty_params.iter().map(generic_param_to_arg);

    Ok(parse_quote! {
        #[automatically_derived]
        impl<#ty_params> ::seu::core::datatypes::DataType for #name<#(#ty_args),*>
        where
            #where_clause
        {
            const TYPE_NAME: &'static str = #name_lit;

            type Repr = #repr;

            type ReprRef<'a> = #repr_ref
            where
                Self: 'a;

            type ReprMut<'a> = #repr_mut
            where
                Self: 'a;

            #[inline]
            fn into_repr(self) -> Self::Repr {
                #to_impl
            }

            #[inline]
            fn as_repr(&self) -> Self::ReprRef<'_> {
                #to_impl_ref
            }

            #[inline]
            fn as_repr_mut(&mut self) -> Self::ReprMut<'_> {
                #to_impl_mut
            }

            #[inline]
            fn from_repr(repr: Self::Repr) -> Self {
                #from_impl
            }
        }
    })
}
