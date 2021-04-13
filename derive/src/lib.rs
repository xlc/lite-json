// Heavily inspred by parity-scale-codec

#![recursion_limit = "128"]
extern crate proc_macro;

#[macro_use]
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro2::{Ident, Span};
use proc_macro_crate::crate_name;
use syn::{DeriveInput, Error};

mod from_json;
mod into_json;
mod trait_bounds;
mod utils;

/// Include the `lite-json` crate under a known name (`_lite_json`).
fn include_crate() -> proc_macro2::TokenStream {
    // This "hack" is required for the tests.
    if std::env::var("CARGO_PKG_NAME").unwrap() == "lite-json" {
        quote!(
            extern crate lite_json as _lite_json;
        )
    } else {
        match crate_name("lite-json") {
            Ok(lite_json_crate) => {
                let ident = Ident::new(&lite_json_crate, Span::call_site());
                quote!( extern crate #ident as _lite_json; )
            }
            Err(e) => Error::new(Span::call_site(), &e).to_compile_error(),
        }
    }
}

/// Wraps the impl block in a "dummy const"
fn wrap_with_dummy_const(impl_block: proc_macro2::TokenStream) -> proc_macro::TokenStream {
    let crate_name = include_crate();

    let generated = quote! {
        const _: () = {
            #[allow(unknown_lints)]
            #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
            #[allow(rust_2018_idioms)]
            #crate_name

            #[cfg(feature = "std")]
            mod __core {
                pub use ::core::*;
                pub use ::std::{vec, vec::Vec};
            }

            #[cfg(not(feature = "std"))]
            mod __core {
                pub use ::core::*;
                pub use ::alloc::{vec, vec::Vec};
            }

            #impl_block
        };
    };

    generated.into()
}

/// Derive `lite_json::IntoJson` and for struct and enum.
#[proc_macro_derive(IntoJson, attributes(json))]
pub fn into_json_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input: DeriveInput = match syn::parse(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error().into(),
    };

    if let Err(e) = utils::check_attributes(&input) {
        return e.to_compile_error().into();
    }

    if let Err(e) = trait_bounds::add(
        &input.ident,
        &mut input.generics,
        &input.data,
        parse_quote!(_lite_json::IntoJson>),
        None,
        utils::get_dumb_trait_bound(&input.attrs),
    ) {
        return e.to_compile_error().into();
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = into_json::quote(&input.data, name);

    let impl_block = quote! {
        impl #impl_generics _lite_json::IntoJson for #name #ty_generics #where_clause {
            fn into_json(self) -> JsonValue {
                #body
            }
        }
    };

    wrap_with_dummy_const(impl_block)
}

/// Derive `lite_json::FromJson` and for struct and enum.
#[proc_macro_derive(Decode, attributes(json))]
pub fn from_json_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input: DeriveInput = match syn::parse(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error().into(),
    };

    if let Err(e) = utils::check_attributes(&input) {
        return e.to_compile_error().into();
    }

    if let Err(e) = trait_bounds::add(
        &input.ident,
        &mut input.generics,
        &input.data,
        parse_quote!(_lite_json::FronJson),
        Some(parse_quote!(Default)),
        utils::get_dumb_trait_bound(&input.attrs),
    ) {
        return e.to_compile_error().into();
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let input_ = quote!(input);
    let body = from_json::quote(&input.data, name, &input_);

    let impl_block = quote! {
        impl #impl_generics _lite_json::FromJson for #name #ty_generics #where_clause {
            fn from_json(input: JsonValue) -> Option<Self> {
                #body
            }
        }
    };

    wrap_with_dummy_const(impl_block)
}
