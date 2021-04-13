// Heavily inspred by http://github.com/paritytech/parity-scale-codec

use proc_macro2::{Ident, Span, TokenStream};
use syn::{spanned::Spanned, Data, Error, Field, Fields};

use crate::utils;

pub fn quote(data: &Data, type_name: &Ident, input: &TokenStream) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(_) | Fields::Unnamed(_) => {
                create_instance(quote! { #type_name }, input, &data.fields)
            }
            Fields::Unit => {
                quote_spanned! { data.fields.span() =>
                    Ok(#type_name)
                }
            }
        },
        Data::Enum(ref data) => {
            let data_variants = || {
                data.variants
                    .iter()
                    .filter(|variant| crate::utils::get_skip(&variant.attrs).is_none())
            };

            let recurse = data_variants().enumerate().map(|(i, v)| {
                let name = &v.ident;
                let index = utils::index(v, i);

                let create = create_instance(quote! { #type_name :: #name }, input, &v.fields);

                quote_spanned! { v.span() =>
                    x if x == #index as u8 => {
                        #create
                    },
                }
            });

            // TODO: match string name

            quote! {
                match #input {
                    _lite_json::JsonValue::Number(_lite_json::NumberValue { integer, .. }) => match integer {
                        #( #recurse )*,
                        _ => None
                    },
                    _ => None,
                }
            }
        }
        Data::Union(_) => {
            Error::new(Span::call_site(), "Union types are not supported.").to_compile_error()
        }
    }
}

fn create_decode_expr(field: &Field, _name: &str, input: &TokenStream) -> TokenStream {
    let skip = utils::get_skip(&field.attrs).is_some();

    if skip {
        quote_spanned! { field.span() => Default::default() }
    } else {
        quote_spanned! { field.span() =>
            _lite_json::FromJson::from_json(#input)?;
        }
    }
}

fn create_instance(name: TokenStream, input: &TokenStream, fields: &Fields) -> TokenStream {
    match *fields {
        Fields::Named(ref fields) => {
            let recurse = fields.named.iter().map(|f| {
                let name_ident = &f.ident;
                let field = match name_ident {
                    Some(a) => format!("{}.{}", name, a),
                    None => format!("{}", name),
                };
                let decode = create_decode_expr(f, &field, input);

                quote_spanned! { f.span() =>
                    #name_ident: #decode
                }
            });

            quote_spanned! { fields.span() =>
                Ok(#name {
                    #( #recurse, )*
                })
            }
        }
        Fields::Unnamed(ref fields) => {
            let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                let name = format!("{}.{}", name, i);

                create_decode_expr(f, &name, input)
            });

            quote_spanned! { fields.span() =>
                Ok(#name (
                    #( #recurse, )*
                ))
            }
        }
        Fields::Unit => {
            quote_spanned! { fields.span() =>
                Ok(#name)
            }
        }
    }
}
