// Heavily inspred by http://github.com/paritytech/parity-scale-codec

use std::str::from_utf8;

use proc_macro2::{Ident, Span, TokenStream};
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, Data, Error, Field, Fields};

use crate::utils;

type FieldsList = Punctuated<Field, Comma>;

fn encode_named_fields<F>(fields: &FieldsList, field_name: F) -> TokenStream
where
    F: Fn(usize, &Option<Ident>) -> TokenStream,
{
    let recurse = fields.iter().enumerate().map(|(i, f)| {
        let skip = utils::get_skip(&f.attrs).is_some();
        let field = field_name(i, &f.ident);

        if skip {
            quote! {}
        } else {
            quote_spanned! { f.span() =>
                (
                    stringify!(f.indent.unwrap()).into_iter().collect(),
                    _lite_json::IntoJson::into_json(#field)
                )
            }
        }
    });

    quote! {
        _lite_json::JsonValue::Object( __core::vec![ #( #recurse, )* ] )
    }
}

fn encode_unnamed_fields<F>(fields: &FieldsList, field_name: F) -> TokenStream
where
    F: Fn(usize, &Option<Ident>) -> TokenStream,
{
    let recurse = fields.iter().enumerate().map(|(i, f)| {
        let skip = utils::get_skip(&f.attrs).is_some();
        let field = field_name(i, &f.ident);

        if skip {
            quote! {}
        } else {
            quote_spanned! { f.span() =>
                _lite_json::IntoJson::into_json(#field)
            }
        }
    });

    quote! {
        _lite_json::JsonValue::Array( __core::vec![ #( #recurse, )* ] )
    }
}

pub fn quote(data: &Data, type_name: &Ident) -> TokenStream {
    let self_ = quote!(self);
    let dest = &quote!(dest);
    let encoding = match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                encode_named_fields(&fields.named, |_, name| quote!(&#self_.#name))
            }
            Fields::Unnamed(ref fields) => encode_unnamed_fields(&fields.unnamed, |i, _| {
                let i = syn::Index::from(i);
                quote!(&#self_.#i)
            }),
            Fields::Unit => quote! {
                _lite_json::JsonValue::Object( __core::vec![] )
            },
        },
        Data::Enum(ref data) => {
            let data_variants = || {
                data.variants
                    .iter()
                    .filter(|variant| crate::utils::get_skip(&variant.attrs).is_none())
            };

            // If the enum has no variants, make it null
            if data_variants().count() == 0 {
                return quote!(_lite_json::JsonValue::Null);
            }

            let recurse = data_variants().map(|f| {
				let name = &f.ident;

				match f.fields {
					Fields::Named(ref fields) => {
						let field_name = |_, ident: &Option<Ident>| quote!(#ident);
						let names = fields.named
							.iter()
							.enumerate()
							.map(|(i, f)| field_name(i, &f.ident));

						let encode_fields = encode_named_fields(
                            &fields.named,
                            |a, b| field_name(a, b),
                        );

						quote_spanned! { f.span() =>
							#type_name :: #name { #( ref #names, )* } => {
                                _lite_json::JsonValue::Array(__core::vec![
                                    _lite_json::JsonValue::String(stringify!(#name).into_iter().collect()),
                                    #encode_fields
                                ])
							}
						}
					},
					Fields::Unnamed(ref fields) => {
						let field_name = |i, _: &Option<Ident>| {
							let data = stringify(i as u8);
							let ident = from_utf8(&data).expect("We never go beyond ASCII");
							let ident = Ident::new(ident, Span::call_site());
							quote!(#ident)
						};
						let names = fields.unnamed
							.iter()
							.enumerate()
							.map(|(i, f)| field_name(i, &f.ident));

						let encode_fields = encode_unnamed_fields(
							&fields.unnamed,
							|a, b| field_name(a, b),
						);

						quote_spanned! { f.span() =>
							#type_name :: #name { #( ref #names, )* } => {
                                _lite_json::JsonValue::Array(__core::vec![
                                    _lite_json::JsonValue::String(stringify!(#name).into_iter().collect()),
                                    #encode_fields
                                ])
							}
						}
					},
					Fields::Unit => {
						quote_spanned! { f.span() =>
							#type_name :: #name => {
								_lite_json::JsonValue::String(stringify!(#name).into_iter().collect())
							}
						}
					},
				}
			});

            quote! {
                match *#self_ {
                    #( #recurse )*,
                    _ => (),
                }
            }
        }
        Data::Union(ref data) => {
            Error::new(data.union_token.span(), "Union types are not supported.").to_compile_error()
        }
    };

    quote! {
        fn encode_to<EncOut: _parity_scale_codec::Output>(&#self_, #dest: &mut EncOut) {
            #encoding
        }
    }
}

pub fn stringify(id: u8) -> [u8; 2] {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    let len = CHARS.len() as u8;
    let symbol = |id: u8| CHARS[(id % len) as usize];
    let a = symbol(id);
    let b = symbol(id / len);

    [a, b]
}
