use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Comma,
    Data, DeriveInput, Ident,
};

#[derive(Debug)]
struct DatashardInfo {
    models: Vec<Ident>,
}

impl DatashardInfo {
    fn new(models: Vec<Ident>) -> Self {
        Self { models }
    }
}

impl Parse for DatashardInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut models = Vec::new();
        loop {
            match input.parse() {
                Ok(ty) => models.push(ty),
                Err(..) => return Ok(DatashardInfo::new(models)),
            };

            match input.parse::<Comma>() {
                Ok(..) => {}
                Err(..) => return Ok(DatashardInfo::new(models)),
            };
        }
    }
}

#[proc_macro]
/// Creates a database struct
pub fn data_shard(input: TokenStream) -> TokenStream {
    let data_shard_info = parse_macro_input!(input as DatashardInfo);
    println!("{data_shard_info:?}");

    let fields = data_shard_info.models.iter().map(|ty| {
        let identifier = format_ident!("{}_repo", ty.to_string().to_lowercase());
        quote! {
            #identifier: dblib::Repository<#ty>,
        }
    });

    let fields_init = data_shard_info.models.iter().map(|ty| {
        let identifier = format_ident!("{}_repo", ty.to_string().to_lowercase());
        quote! {
            #identifier: dblib::Repository::new(),
        }
    });

    let methods = data_shard_info.models.clone();
    let methods = methods
        .iter()
        .map(|ty| format_ident!("get_{}", ty.to_string().to_lowercase()))
        .zip(data_shard_info.models.iter());

    let insert = data_shard_info.models.iter().map(|ty| {
        let ident = format_ident!("insert_{}", ty.to_string().to_lowercase());
        let repo = format_ident!("{}_repo", ty.to_string().to_lowercase());

        quote! {
            pub fn #ident (&mut self, value: #ty) {
                self.#repo.insert_one(value);
            }
        }
    });

    let branches = methods.clone().map(|(service, ..)| {
        quote! {
            Service::#service(#service) => #service.register(a_s)
        }
    });

    let services = methods.clone().map(|(service, ..)| {
        quote! {
            #service(#service)
        }
    });

    let services_list = methods.clone().map(|(service, ..)| {
        quote! {
            .service(#service)
        }
    });

    let endpoints = methods.clone().map(|(ident, struct_name)| {
        let method_name = ident.to_string();
        let query_params = format_ident!("{}QueryParams", struct_name);
        let repo = format_ident!("{}_repo", struct_name.to_string().to_lowercase());

        quote! {
            #[dblib::actix_web::get(#method_name)]
            pub async fn #ident(
                db: dblib::actix_web::web::Data<std::sync::Arc<dblib::futures::lock::Mutex<DataShard>>>,
                query: dblib::actix_web::web::Query<#query_params>,
                params: dblib::actix_web::web::Query<dblib::QueryParams>,
            ) -> dblib::actix_web::Result<impl dblib::actix_web::Responder> {
                let db = db.clone();
                let db = db.lock().await;

                let builder = db
                    .#repo
                    .filter_builder()
                    .filter(|doc| doc.matches_criteria(&query));

                if let Some(limit) = params.0.limit {
                    let builder = builder.take(limit);
                    return Ok(dblib::serde_json::to_string(&builder.collect::<Vec<_>>()))
                }

                Ok(dblib::serde_json::to_string(&builder.collect::<Vec<_>>()))
            }
        }
    });

    let res = quote! {
        #(#endpoints)*

        #[derive(Clone)]
        pub struct DataShard {
            id: dblib::RID,
            #(#fields),*
        }

        pub enum Service {
            #(#services),*
        }

        impl dblib::actix_web::dev::HttpServiceFactory for Service {
            fn register(self, a_s: &mut dblib::actix_web::dev::AppService) {
                match self {
                    #(#branches)*,
                }
            }
        }

        impl DataShard {
            pub fn new(id: impl ToString) -> Self {
                Self {
                    id: dblib::RID::new(id),
                    #(#fields_init),*
                }
            }

            #(#insert)*


            pub fn setup(config: &mut dblib::actix_web::web::ServiceConfig) {
                config #(#services_list)*;
                // TODO: Implement data loading
                // TODO: Implement non-static id
                let db = std::sync::Arc::new(dblib::futures::lock::Mutex::new(DataShard::new("test")));

                config.app_data(dblib::actix_web::web::Data::new(db.clone()));
                println!("Setting up!");
            }
        }

        #[no_mangle]
        pub fn setup_shard() -> fn(&mut dblib::actix_web::web::ServiceConfig) {
            println!("Call!");
            DataShard::setup
        }
    };

    println!("{:#}", res.to_string());

    res.into()
}

#[proc_macro_derive(QueryParams)]
/// Creates new structs and methods to make filtering easier
/// QueryParams creates a struct with optional fields so that it can
/// either be used with actix query params or with the builder
pub fn query_params(input: TokenStream) -> TokenStream {
    let DeriveInput { data, ident, .. } = parse_macro_input!(input as DeriveInput);

    let data = match data {
        Data::Struct(data) => data,
        _ => return quote!(compile_error!("#[derive(QueryParams)] only works on structs");).into(),
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
        let type_token = field.ty.clone().into_token_stream();

        quote! {
            pub fn #field_ident(mut self, value: impl Into<#type_token>) -> Self {
                self.0.#field_ident = Some(value.into());
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
        #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
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
