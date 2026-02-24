//! Options derive macro implementation.

mod docs;
mod generate;
mod parse;

use docs::generate_doc_metadata;
use generate::{
    generate_default_impl, generate_merge_impl, generate_partial_fields, generate_resolve_impl,
};
use parse::{
    ParsedField, extract_named_fields, get_partial_name, parse_field, parse_struct_options,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DeriveInput;

/// Implement the Options derive macro.
pub(crate) fn derive(input: DeriveInput) -> syn::Result<TokenStream2> {
    let struct_name = &input.ident;
    let struct_opts = parse_struct_options(&input.attrs)?;
    let partial_name = get_partial_name(&struct_opts, struct_name);
    let fields = extract_named_fields(&input)?;
    let parsed_fields: Vec<ParsedField> =
        fields.iter().map(parse_field).collect::<syn::Result<_>>()?;
    let partial_fields = generate_partial_fields(&parsed_fields);
    let merge_impl = generate_merge_impl(&parsed_fields);
    let resolve_impl =
        generate_resolve_impl(struct_name, &partial_name, &parsed_fields, &struct_opts);
    let resolve_inherent = resolve_impl.inherent;
    let resolve_trait_methods = resolve_impl.trait_methods;
    let default_impl = generate_default_impl(struct_name, &parsed_fields);
    let doc_metadata_impl = generate_doc_metadata(struct_name, &input.attrs, &parsed_fields);
    Ok(quote! {
        /// Partial options struct with optional fields for CLI/YAML parsing.
        #[derive(::clap::Args, ::std::clone::Clone, ::std::fmt::Debug, ::std::default::Default, ::serde::Deserialize, ::serde::Serialize)]
        pub struct #partial_name {
            #partial_fields
        }
        #resolve_inherent
        impl crate::options::OptionsPartialContract for #partial_name {
            type Resolved = #struct_name;
            #merge_impl
            #resolve_trait_methods
        }
        #default_impl
        #doc_metadata_impl
        const _: () = {
            fn __register(
                provider: &mut crate::options::OptionsProvider,
                services: &mut ::di::ServiceCollection,
            ) {
                provider.register::<#partial_name>(services);
            }
            ::inventory::submit!(crate::options::OptionsRegistration {
                doc_metadata: <#struct_name as crate::options::Documented>::doc_metadata,
                register: __register,
            });
        };
    })
}
