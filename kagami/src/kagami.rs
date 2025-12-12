// custom_model.rs

use darling::{FromDeriveInput, FromMeta, util::PathList};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataStruct, Field, Fields};

#[derive(FromDeriveInput, Clone)]
#[darling(attributes(kagami), supports(struct_named))]
pub(crate) struct Kagami {
    pub interned_name: syn::Ident,
    pub interned_derives: PathList,
    pub persisted_name: syn::Ident,
    pub persisted_derives: PathList,
    pub interned_context: syn::Path,
    pub persisted_context: syn::Path,
    #[darling(default)]
    pub shared: PathList,
}

pub(crate) fn refract_persistent(
    structure: &DataStruct,
    original_name: &str, // interned object
    name: &str,          // persisted object
    context: &syn::Path,
    private_derives: &PathList,
    shared_derives: &PathList,
) -> TokenStream {
    // generate definition
    let mut contents = quote!();

    for field in structure.fields.iter() {
        let field_type = &field.ty;
        let identifier = field.ident.as_ref().unwrap();

        contents.extend(
            quote! { #identifier : <<#field_type as Internable<#context>>::Interned as ::hone::node::Persistent<#context>>::Persisted},
        );
    }

    let definition = quote! {
        #[derive(#(#private_derives),*, #(#shared_derives),*)]
        pub struct #name{
            #contents
        }
    };

    // generate implement for Internable
    let mut to_persisted = quote!();
    let mut from_persisted = quote!();

    for field in structure.fields.iter() {
        let field_type = &field.ty;
        let identifier = field.ident.as_ref().unwrap();

        to_persisted.extend(quote! {
            #identifier : #<<#field_type as ::zako_core::intern::Internable<#context>>::Interned as ::hone::node::Persistent<#context>>::to_persisted(&self.#identifier, __context),
        });
        from_persisted.extend(quote! {
            #identifier : #<<#field_type as ::zako_core::intern::Internable<#context>>::Interned as ::hone::node::Persistent<#context>>::from_persisted(&__context, __persisted.#identifier),
        });
    }

    let persistent_impl = quote! {
        impl ::hone::node::Persistent<#context> for #original_name {
            type Persisted = #name;
            fn to_persisted(&self,__context: &Self::Context) -> Self::Persisted {
                Self::Persisted{
                    #to_persisted
                }
            }
            fn from_persisted(__context: &Self::Context, __persisted: Self::Persisted) -> Self {
                Self{
                    #from_persisted
                }
            }
        }
    };

    quote! {
        #definition
        #persistent_impl
    }
    .into()
}

pub(crate) fn refract_internable(
    structure: &DataStruct,
    original_name: &str, // object
    name: &str,          // interned object
    context: &syn::Path,
    private_derives: &PathList,
    shared_derives: &PathList,
) -> TokenStream {
    // generate definition
    let mut contents = quote!();

    for field in structure.fields.iter() {
        let field_type = &field.ty;
        let identifier = field.ident.as_ref().unwrap();

        contents.extend(quote! { #identifier : <#field_type as ::zako_core::intern::Internable<#context>>::Interned});
    }

    let definition = quote! {
        #[derive(#(#private_derives),*, #(#shared_derives),*)]
        pub struct #name{
            #contents
        }
    };

    // generate implement for Internable
    let mut intern = quote!();
    let mut deintern = quote!();

    for field in structure.fields.iter() {
        let field_type = &field.ty;
        let identifier = field.ident.as_ref().unwrap();

        intern.extend(quote! {
            #identifier : #field_type::intern(&self.#identifier, __context),
        });
        deintern.extend(quote! {
            #identifier : #field_type::deintern(__interned.#identifier,&__context),
        });
    }

    let internable_impl = quote! {
        impl ::zako_core::intern::Internable<#context> for #original_name {
            type Interned = #name;
            fn intern(self,__context: &#context) -> Self::Interned {
                Self::Interned{
                    #intern
                }
            }
            fn deintern(__interned: Self::Interned,__context: &#context,) -> Self {
                Self{
                    #deintern
                }
            }
        }
    };

    quote! {
        #definition
        #internable_impl
    }
    .into()
}
