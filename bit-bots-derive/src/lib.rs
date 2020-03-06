extern crate proc_macro;
use crate::proc_macro::TokenStream;
use heck::SnekCase;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(SerializableComponent)]
pub fn serializable_component_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    // get the name of the type we want to implement the trait for
    let name = &input.ident;
    let string_name = format!("{}", name).to_snek_case();

    let expanded = quote! {
        impl crate::components::SerializableComponent for #name {
            const SERIALIZATION_NAME: once_cell::sync::Lazy<serde_yaml::Value> =
                once_cell::sync::Lazy::new(|| serde_yaml::Value::String(#string_name.to_owned()));
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(NonSceneGraphComponent)]
pub fn non_scene_graph_component_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    // get the name of the type we want to implement the trait for
    let name = &input.ident;

    let expanded = quote! {
        impl crate::components::NonSceneGraphComponent for #name {}
    };

    TokenStream::from(expanded)
}