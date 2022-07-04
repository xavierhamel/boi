use proc_macro;
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, DeriveInput, Ident, Lit, Meta, NestedMeta, Variant};

// enum Typ {
//     #[boi_typ(code = 0b00, use_channels, size = 3)]
//     Short,
// }
//
// impl From<usize> for Typ {
//     fn from(code: usize) -> Self {
//         match code {
//             0b00 => Self::Short,
//             0b01 => Self::Medium,
//             0b11 => Self::Long,
//             _ => panic!("The block with the code `{code}` does not exists"),
//         }
//     }
// }
//
// impl Typ {
//     pub fn size(&self) -> usize {
//         match code {
//             0b00 => Self::Short,
//             0b01 => Self::Medium,
//             0b11 => Self::Long,
//             _ => panic!("The block with the code `{code}` does not exists"),
//         }
//     }
// }

#[proc_macro_derive(BoiTyp, attributes(boi))]
pub fn boi_typ_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (code_body, size_body, consts) = match input.data {
        syn::Data::Enum(data) => parse_variants(data.variants),
        _ => panic!("Should be an enum"),
    };

    let expanded = quote! {
        impl<const CHANNELS: usize> #name<CHANNELS> {
            #consts

            pub fn size(&self) -> usize {
                match self {
                    #size_body
                }
            }
        }

        impl<const CHANNELS: usize> From<usize> for #name<CHANNELS> {
            fn from(code: usize) -> Self {
                match code {
                    #code_body,
                    _ => panic!("The code does not exists")
                }
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn parse_variants(variants: Punctuated<Variant, Comma>) -> (TokenStream, TokenStream, TokenStream) {
    let mut attrs = Vec::new();
    for variant in variants.iter() {
        for attr in variant.attrs.iter() {
            if attr.path.is_ident("boi") {
                attrs.push((variant.ident.clone(), attr.parse_meta()));
            }
        }
    }

    let mut code_bodies = Vec::new();
    let mut size_bodies = Vec::new();
    let mut consts: Vec<TokenStream> = Vec::new();
    for (ident, meta) in attrs.iter() {
        let upper_ident = &ident.to_string().to_uppercase();
        let const_code_ident = Ident::new(&format!("{upper_ident}_CODE"), ident.span());
        let const_code_len_ident = Ident::new(&format!("{upper_ident}_CODE_LEN"), ident.span());
        let const_size_ident = Ident::new(&format!("{upper_ident}_SIZE"), ident.span());
        if let Ok(meta) = meta {
            match meta {
                Meta::List(meta_list) => {
                    let (is_using_channels, code, code_len, size) = parse_attrs(meta_list);
                    consts.push(quote! {
                        const #const_code_ident: usize = #code;
                        const #const_code_len_ident: usize = #code_len;
                        const #const_size_ident: usize = #size;
                    });
                    code_bodies.push(quote! {
                        #code => Self::#ident
                    });
                    if is_using_channels {
                        size_bodies.push(quote! {
                            Self::#ident => CHANNELS * #size + #code_len
                        });
                    } else {
                        size_bodies.push(quote! {
                            Self::#ident => #size + #code_len
                        });
                    }
                }
                _ => {}
            }
        }
    }
    (
        quote! { #(#code_bodies),* },
        quote! { #(#size_bodies),* },
        quote! { #(#consts)* },
    )
}

fn parse_attrs(meta_list: &syn::MetaList) -> (bool, usize, usize, usize) {
    let mut code = 0;
    let mut size = 0;
    let mut code_len = 0;
    let mut is_using_channels = false;
    meta_list.nested.iter().for_each(|nested| match nested {
        NestedMeta::Meta(m) => match m {
            Meta::NameValue(name_value) => {
                match &name_value.lit {
                    Lit::Int(int) => {
                        let value = int.base10_parse::<usize>().unwrap();
                        if name_value.path.is_ident("code") {
                            code = value;
                        } else if name_value.path.is_ident("size") {
                            size = value;
                        } else if name_value.path.is_ident("code_len") {
                            code_len = value;
                        }
                    }
                    Lit::Bool(_) => {
                        if name_value.path.is_ident("uses_channels") {
                            is_using_channels = true;
                        }
                    }
                    _ => {}
                };
            }
            _ => unreachable!(),
        },
        _ => {}
    });
    (is_using_channels, code, code_len, size)
}
