// Taken from http://github.com/paritytech/parity-scale-json

// Copyright 2018 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Various internal utils.
//!
//! NOTE: attributes finder must be checked using check_attribute first, otherwise macro can panic.

use proc_macro2::{Span, TokenStream};
use syn::{
    spanned::Spanned, Attribute, Data, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Lit, Meta,
    MetaNameValue, NestedMeta, Variant,
};

fn find_meta_item<'a, F, R, I>(itr: I, pred: F) -> Option<R>
where
    F: FnMut(&NestedMeta) -> Option<R> + Clone,
    I: Iterator<Item = &'a Attribute>,
{
    itr.filter_map(|attr| {
        if attr.path.is_ident("json") {
            if let Meta::List(ref meta_list) = attr
                .parse_meta()
                .expect("Internal error, parse_meta must have been checked")
            {
                return meta_list.nested.iter().filter_map(pred.clone()).next();
            }
        }

        None
    })
    .next()
}

pub fn index(v: &Variant, i: usize) -> TokenStream {
    // look for an index in attributes
    let index = find_meta_item(v.attrs.iter(), |meta| {
        if let NestedMeta::Meta(Meta::NameValue(ref nv)) = meta {
            if nv.path.is_ident("index") {
                if let Lit::Int(ref v) = nv.lit {
                    let byte = v
                        .base10_parse::<u8>()
                        .expect("Internal error, index attribute must have been checked");
                    return Some(byte);
                }
            }
        }

        None
    });

    // then fallback to discriminant or just index
    index.map(|i| quote! { #i }).unwrap_or_else(|| {
        v.discriminant
            .as_ref()
            .map(|&(_, ref expr)| quote! { #expr })
            .unwrap_or_else(|| quote! { #i })
    })
}

// return span of skip if found
pub fn get_skip(attrs: &[Attribute]) -> Option<Span> {
    // look for `skip` in the attributes
    find_meta_item(attrs.iter(), |meta| {
        if let NestedMeta::Meta(Meta::Path(ref path)) = meta {
            if path.is_ident("skip") {
                return Some(path.span());
            }
        }

        None
    })
}

/// Returns if the `dumb_trait_bound` attribute is given in `attrs`.
pub fn get_dumb_trait_bound(attrs: &[Attribute]) -> bool {
    find_meta_item(attrs.iter(), |meta| {
        if let NestedMeta::Meta(Meta::Path(ref path)) = meta {
            if path.is_ident("dumb_trait_bound") {
                return Some(());
            }
        }

        None
    })
    .is_some()
}

pub fn check_attributes(input: &DeriveInput) -> syn::Result<()> {
    for attr in &input.attrs {
        check_top_attribute(attr)?;
    }

    match input.data {
        Data::Struct(ref data) => match &data.fields {
            Fields::Named(FieldsNamed { named: fields, .. })
            | Fields::Unnamed(FieldsUnnamed {
                unnamed: fields, ..
            }) => {
                for field in fields {
                    for attr in &field.attrs {
                        check_field_attribute(attr)?;
                    }
                }
            }
            Fields::Unit => (),
        },
        Data::Enum(ref data) => {
            for variant in data.variants.iter() {
                for attr in &variant.attrs {
                    check_variant_attribute(attr)?;
                }
                for field in &variant.fields {
                    for attr in &field.attrs {
                        check_field_attribute(attr)?;
                    }
                }
            }
        }
        Data::Union(_) => (),
    }
    Ok(())
}

// Is accepted only:
// * `#[json(skip)]`
fn check_field_attribute(attr: &Attribute) -> syn::Result<()> {
    let field_error = "Invalid attribute on field, only `#[json(skip)]` is accepted.";

    if attr.path.is_ident("json") {
        match attr.parse_meta()? {
            Meta::List(ref meta_list) if meta_list.nested.len() == 1 => {
                match meta_list.nested.first().unwrap() {
                    NestedMeta::Meta(Meta::Path(path))
                        if path.get_ident().map_or(false, |i| i == "skip") =>
                    {
                        Ok(())
                    }
                    elt @ _ => Err(syn::Error::new(elt.span(), field_error)),
                }
            }
            meta @ _ => Err(syn::Error::new(meta.span(), field_error)),
        }
    } else {
        Ok(())
    }
}

// Is accepted only:
// * `#[json(skip)]`
// * `#[json(index = $int)]`
fn check_variant_attribute(attr: &Attribute) -> syn::Result<()> {
    let variant_error = "Invalid attribute on variant, only `#[json(skip)]` and \
		`#[json(index = $u8)]` are accepted.";

    if attr.path.is_ident("json") {
        match attr.parse_meta()? {
            Meta::List(ref meta_list) if meta_list.nested.len() == 1 => {
                match meta_list.nested.first().unwrap() {
                    NestedMeta::Meta(Meta::Path(path))
                        if path.get_ident().map_or(false, |i| i == "skip") =>
                    {
                        Ok(())
                    }

                    NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                        path,
                        lit: Lit::Int(lit_int),
                        ..
                    })) if path.get_ident().map_or(false, |i| i == "index") => lit_int
                        .base10_parse::<u8>()
                        .map(|_| ())
                        .map_err(|_| syn::Error::new(lit_int.span(), "Index must be in 0..255")),

                    elt @ _ => Err(syn::Error::new(elt.span(), variant_error)),
                }
            }
            meta @ _ => Err(syn::Error::new(meta.span(), variant_error)),
        }
    } else {
        Ok(())
    }
}

// Only `#[json(dumb_trait_bound)]` is accepted as top attribute
fn check_top_attribute(attr: &Attribute) -> syn::Result<()> {
    let top_error = "Invalid attribute only `#[json(dumb_trait_bound)]` is accepted as top \
		attribute";
    if attr.path.is_ident("json") {
        match attr.parse_meta()? {
            Meta::List(ref meta_list) if meta_list.nested.len() == 1 => {
                match meta_list.nested.first().unwrap() {
                    NestedMeta::Meta(Meta::Path(path))
                        if path.get_ident().map_or(false, |i| i == "dumb_trait_bound") =>
                    {
                        Ok(())
                    }

                    elt @ _ => Err(syn::Error::new(elt.span(), top_error)),
                }
            }
            meta @ _ => Err(syn::Error::new(meta.span(), top_error)),
        }
    } else {
        Ok(())
    }
}
