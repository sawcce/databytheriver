use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, token::Struct, Data, DeriveInput};

#[proc_macro_derive(QueryParams)]
pub fn query_params(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs, data, ident, ..
    } = parse_macro_input!(input as DeriveInput);

    let data = match data {
        Data::Struct(data) => data,
        _ => {
            println!("Only supported on structs!");
            return TokenStream::new();
        }
    };

    let fields = data.fields.iter().map(|field| {
        let field_ident = field.ident.clone().into_token_stream();
        let type_token = field.ty.clone().into_token_stream();
        quote! {
            #field_ident: Option<#type_token>
        }
    });

    let default_values = data.fields.iter().map(|field| {
        let field_ident = field.ident.clone().into_token_stream();
        quote! {
            #field_ident: None,
        }
    });

    let methods = data.fields.iter().map(|field| {
        let field_ident = field.ident.clone().into_token_stream();

        quote! {
            pub fn #field_ident(mut self, value: String) -> Self {
                self.0.#field_ident = Some(value);
                self
            }
        }
    });

    let checks = data.fields.iter().map(|field| {
        let field_ident = field.ident.clone().into_token_stream();
        quote! {
            match query.#field_ident {
                Some(ref value) => {
                    if *value != self.#field_ident {
                        return false;
                    }
                },
                None => {}
            };
        }
    });

    let structname = format_ident!("{}QueryParams", ident);
    let builder = format_ident!("{}QueryParamsBuilder", ident);

    let result = quote! {
        #[derive(serde::Deserialize, Clone, Debug)]
        pub struct #structname {
            #(#fields),*
        }

        impl #structname {
            pub fn default() -> Self {
                Self {
                    #(#default_values)*
                }
            }

            pub fn builder() -> #builder {
                #builder::new()
            }
        }

        pub struct #builder (#structname);

        impl #builder {
            pub fn new() -> Self {
                Self(#structname::default())
            }

            pub fn wrap(self) -> #structname {
                self.0
            }

            #(#methods)*
        }

        impl #ident {
            pub fn matches_criteria(&self, query: &#structname) -> bool {
                #(#checks)*
                return true;
            }
        }
    }
    .into();

    println!("{result:#}");

    result
}
