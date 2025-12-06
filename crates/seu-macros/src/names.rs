//! Type-level name generation for fields and enum variants
//!
//! The generated representation types use Rust identifiers at the type level. For each distinct
//! identifier passed to [`NameGen::insert`], this module emits a zero-sized marker type named
//! `<Ident>Name` and an implementation of the configured `HasName` trait for that marker type.
//! The implementation exposes the original Rust identifier as a string constant.
//!
//! Reusing the same identifier returns the same marker type and emits a single declaration, which
//! lets a derive macro freely request names while walking nested fields and variants.

use std::borrow::Borrow;
use std::collections::HashMap;

use quote::format_ident;
use syn::Fields;
use syn::Ident;
use syn::Item;
use syn::ItemImpl;
use syn::ItemStruct;
use syn::LitStr;
use syn::Type;
use syn::Variant;
use syn::parse_quote;

/// Generated artifacts for one source identifier
struct Name {
    /// Zero-sized marker type declaration, such as `struct FieldName;`
    name_decl: ItemStruct,
    /// Implementation of `HasName` that binds the marker type to the original identifier string
    impl_decl: ItemImpl,
    /// Type used by generated representation code to refer to the marker
    name: Type,
}

struct NameSeq {
    name_decl: ItemStruct,
    impl_decl: ItemImpl,
    name: Type,
}

/// Collects and de-duplicates generated name marker types
///
/// Use [`NameGen::insert`] while constructing type-level representations, then append the items
/// returned by [`NameGen::done`] to the proc-macro output so every referenced marker type is in
/// scope.
#[derive(Default)]
pub struct NameGen {
    /// Generated names keyed by the source identifier that requested them
    names: HashMap<Ident, Name>,
    /// Generated names keyed by the sequence of source identifiers
    name_seqs: HashMap<Vec<Ident>, NameSeq>,
}

impl NameGen {
    /// Declare a name marker type.
    ///
    /// Subsequent calls with the same identifier return the existing type without adding duplicate
    /// declarations.
    pub fn insert(&mut self, ident: &Ident) -> &Type {
        let entry = self.names.entry(ident.clone()).or_insert_with(|| {
            let struct_ident = format_ident!("{ident}Name");
            let name_lit = LitStr::new(ident.to_string().as_str(), ident.span());

            Name {
                name_decl: parse_quote! {
                    #[automatically_derived]
                    struct #struct_ident;
                },
                impl_decl: parse_quote! {
                    #[automatically_derived]
                    impl ::seu::core::names::HasName for #struct_ident {
                        const NAME: &'static str = #name_lit;
                    }
                },
                name: parse_quote! {
                    #struct_ident
                },
            }
        });

        &entry.name
    }

    /// Declare the names for a product of fields.
    pub fn insert_fields(&mut self, fields: &Fields) -> &Type {
        self.insert_names(fields.iter().filter_map(|field| field.ident.as_ref()))
    }

    /// Declare the names for a product of variants.
    pub fn insert_variants<'v>(
        &mut self,
        variants: impl IntoIterator<Item = &'v Variant>,
    ) -> &Type {
        self.insert_names(variants.into_iter().map(|var| &var.ident))
    }

    fn insert_names<I: Borrow<Ident>>(&mut self, names: impl IntoIterator<Item = I>) -> &Type {
        let (field_name_tys, field_names): (Vec<_>, Vec<_>) = names
            .into_iter()
            .map(|ident| {
                let ident = ident.borrow();
                (self.insert(ident).clone(), ident.clone())
            })
            .unzip();

        let next_idx = self.name_seqs.len();

        let entry = self
            .name_seqs
            .entry(field_names.clone())
            .or_insert_with(|| {
                let ident = format_ident!("__SeuNames_{next_idx}");

                let name_decl = parse_quote! {
                    struct #ident;
                };

                let field_exprs = field_name_tys.into_iter().map(|name_ty| -> Type {
                    parse_quote! {
                        <#name_ty as ::seu::core::names::HasName>::NAME
                    }
                });

                let impl_decl: ItemImpl = parse_quote! {
                    impl ::seu::core::names::HasNames for #ident {
                        const NAMES: &'static [&'static str] = &[
                            #(#field_exprs),*
                        ];
                    }
                };

                let name = parse_quote!(#ident);

                NameSeq {
                    name_decl,
                    impl_decl,
                    name,
                }
            });

        &entry.name
    }

    /// Consumes the generator and yields the declarations required by all inserted names.
    ///
    /// The caller must include these items in the proc-macro output; otherwise representation code
    /// that references the returned marker types will fail to compile.
    pub fn done(self) -> impl Iterator<Item = Item> {
        let names = self
            .names
            .into_values()
            .flat_map(|name| [Item::from(name.name_decl), Item::from(name.impl_decl)]);
        let field_names = self
            .name_seqs
            .into_values()
            .flat_map(|item| [Item::from(item.name_decl), Item::from(item.impl_decl)]);
        names.chain(field_names)
    }
}
