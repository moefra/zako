use darling::{FromDeriveInput, FromField, FromMeta, util::PathList};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataStruct, Field, Fields, token::Type};

// TODO: Remove hard-code paths
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

#[derive(FromField, Clone)]
#[darling(attributes(kagami))]
pub(crate) struct SkipAttribute {
    #[darling(default = ||false)]
    pub skip: bool,
}

impl Default for SkipAttribute {
    fn default() -> Self {
        SkipAttribute { skip: false }
    }
}

pub(crate) fn refract_persistent(
    structure: &DataStruct,
    original_name: &syn::Ident, // interned object
    name: &syn::Ident,          // persisted object
    context: &syn::Path,
    private_derives: &PathList,
    shared_derives: &PathList,
) -> TokenStream {
    // generate definition
    let mut contents = quote!();

    // TODO: Merge two loop into one

    for field in structure.fields.iter() {
        let field_type = &field.ty;
        let identifier = field.ident.as_ref().unwrap();

        let skip_attribute = SkipAttribute::from_field(field).unwrap_or_default();

        if skip_attribute.skip {
            contents.extend(quote! { #identifier : #field_type, });
        } else {
            contents.extend(quote! { #identifier : <<#field_type as ::zako_core::intern::Internable>::Interned as ::hone::node::Persistent>::Persisted, });
        }
    }

    let definition = quote! {
        #[derive(#(#private_derives),*, #(#shared_derives),*)]
        pub struct #name{
            #contents
        }
    };

    // generate implement for Persistent
    let mut to_persisted = quote!();
    let mut from_persisted = quote!();

    for field in structure.fields.iter() {
        let field_type = &field.ty;
        let identifier = field.ident.as_ref().unwrap();

        let skip_attribute = SkipAttribute::from_field(field).unwrap_or_default();

        if skip_attribute.skip {
            to_persisted.extend(quote! {
                #identifier : self.#identifier.clone(),
            });
            from_persisted.extend(quote! {
                #identifier : self.#identifier.clone(),
            });
        } else {
            to_persisted.extend(quote! {
                #identifier : <<#field_type as ::zako_core::intern::Internable>::Interned as ::hone::node::Persistent<#context>>::to_persisted(&self.#identifier, &__context),
            });
            from_persisted.extend(quote! {
                #identifier : <<#field_type as ::zako_core::intern::Internable>::Interned as ::hone::node::Persistent<#context>>::from_persisted(__persisted.#identifier,&__context),
            });
        }
    }

    let persistent_impl = quote! {
        impl ::hone::node::Persistent<#context> for #original_name {
            type Persisted = #name;
            fn to_persisted(&self,__context: &#context) -> Self::Persisted {
                Self::Persisted{
                    #to_persisted
                }
            }
            fn from_persisted(__persisted: Self::Persisted,__context: &#context) -> Self {
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
    original_name: &syn::Ident, // object
    name: &syn::Ident,          // interned object
    context: &syn::Path,
    private_derives: &PathList,
    shared_derives: &PathList,
) -> TokenStream {
    // generate definition
    let mut contents = quote!();

    for field in structure.fields.iter() {
        let field_type = &field.ty;
        let identifier = field.ident.as_ref().unwrap();

        let skip_attribute = SkipAttribute::from_field(field).unwrap_or_default();

        if skip_attribute.skip {
            contents.extend(quote! { #identifier : #field_type, });
        } else {
            contents.extend(quote! { #identifier : <#field_type as ::zako_core::intern::Internable>::Interned, });
        }
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

        let skip_attribute = SkipAttribute::from_field(field).unwrap_or_default();

        if skip_attribute.skip {
            intern.extend(quote! {
                #identifier : self.#identifier,
            });
            deintern.extend(quote! {
                #identifier : self.#identifier,
            });
        } else {
            intern.extend(quote! {
                #identifier : <#field_type as ::zako_core::intern::Internable<#context>>::intern(self.#identifier, &__context),
            });
            deintern.extend(quote! {
                #identifier : <#field_type as ::zako_core::intern::Internable<#context>>::resolve(&__interned.#identifier,&__context),
            });
        }
    }

    let internable_impl = quote! {
        impl ::zako_core::intern::Internable<#context> for #original_name {
            type Interned = #name;
            fn intern(self,__context: &#context) -> Self::Interned {
                Self::Interned{
                    #intern
                }
            }
            fn resolve(__interned: &Self::Interned,__context: &#context) -> Self {
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
