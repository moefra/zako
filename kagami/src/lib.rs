mod kagami;
extern crate proc_macro2;

use darling::FromDeriveInput;
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
                    &struct_identifier,
                    &kagami.interned_name,
                    &kagami.interned_context,
                    &kagami.interned_derives,
                    &kagami.shared,
                ),
                kagami::refract_persistent(
                    &structure,
                    &struct_identifier,
                    &kagami.persisted_name,
                    &kagami.persisted_context,
                    &kagami.persisted_derives,
                    &kagami.shared,
                ),
            )
        }
        Data::Enum(enum_) => todo!(),
        _ => panic!("Unsupported use `kagami` in enum other than struct"),
    };

    quote! {
        #original

        #interned

        #persisted
    }
    .into()
}
