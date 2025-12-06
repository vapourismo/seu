mod datatypes;
mod enums;
mod fields;
mod names;
mod structs;
mod variants;

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::Error;
use syn::Item;
use syn::visit_mut::VisitMut;
use syn::visit_mut::visit_item_mut;
use syn::visit_mut::visit_path_segment_mut;

use crate::derive::datatypes::data_type_impl;
use crate::names::NameGen;
use crate::opts::Options;

/// Implementor for `VisitMut` that fixes hardcoded crate paths
struct FixCrate<'a>(&'a Options);

impl VisitMut for FixCrate<'_> {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        let Some(first) = path.segments.first_mut() else {
            return;
        };

        if first.ident == "seu" {
            let mut new_path = self.0.target_crate.clone();
            new_path
                .segments
                .extend(path.segments.iter().skip(1).cloned());
            *path = new_path;
        }

        // We need to recurse deeper, otherwise me might miss nested path arguments.
        for seg in &mut path.segments {
            visit_path_segment_mut(self, seg);
        }
    }
}

pub fn main(mut input: DeriveInput) -> Result<TokenStream, Error> {
    let mut opts = Options::default();
    opts.load_from(&mut input.attrs)?;

    let mut names = NameGen::default();

    let data_type_impl = data_type_impl(input, &mut names)?;

    let items = names
        .done()
        .chain([Item::from(data_type_impl)])
        .map(|mut item| {
            visit_item_mut(&mut FixCrate(&opts), &mut item);
            item
        });

    Ok(quote! {
        const _: () = {
            #(#items)*
        };
    })
}
