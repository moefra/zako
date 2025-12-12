mod kagami;
extern crate proc_macro2;

use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Data;

use crate::kagami::Kagami;

#[proc_macro_derive(Kagami)]
pub fn inverse_image_through(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let original_ast = syn::parse_macro_input!(item as syn::DeriveInput);

    let original = quote! {#original_ast};

    let struct_identifier = &original_ast.ident;

    let (interned, persisted) = match &original_ast.data {
        Data::Struct(structure) => {
            let kagami = Kagami::from_derive_input(&original_ast).unwrap();

            (
                kagami::refract_internable(
                    &structure,
                    struct_identifier.to_string().as_str(),
                    kagami.interned_name.to_string().as_str(),
                    &kagami.interned_context,
                    &kagami.interned_derives,
                    &kagami.shared,
                ),
                kagami::refract_internable(
                    &structure,
                    kagami.interned_name.to_string().as_str(),
                    kagami.persisted_name.to_string().as_str(),
                    &kagami.persisted_context,
                    &kagami.persisted_derives,
                    &kagami.shared,
                ),
            )
        }
        _ => panic!("Unsupported use `kagami` in enum other than struct"),
    };

    quote! {
        #original

        #interned

        #persisted
    }
    .into()
}
