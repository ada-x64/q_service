use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, parse_quote};

#[proc_macro_derive(ServiceLabel)]
pub fn derive_service_name(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(
            Self: Send + Sync + Clone + PartialEq + Eq + std::fmt::Debug + std::hash::Hash + 'static
        ));

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) =
        &ast.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics q_service::prelude::ServiceLabel for #struct_name #type_generics #where_clause {}
    })
}

#[proc_macro_derive(ServiceError)]
pub fn derive_service_error(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! {
            Self: std::error::Error + Clone + PartialEq + Send + Sync + 'static
        });

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) =
        &ast.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics q_service::prelude::ServiceError for #struct_name #type_generics #where_clause {}
    })
}

#[proc_macro_derive(ServiceData)]
pub fn derive_service_data(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! {
            Self: Clone + std::fmt::Debug + PartialEq + Send + Sync + Default + 'static
        });

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) =
        &ast.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics q_service::prelude::ServiceData for #struct_name #type_generics #where_clause {}
    })
}
