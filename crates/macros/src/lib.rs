//! Derive macro for generating partial options structs.
//!
//! This crate provides the `#[derive(Options)]` macro which generates a "partial"
//! struct variant where all fields are wrapped in `Option<T>`, along with trait
//! implementations for merging, resolution, and validation.
//!
//! # Hygiene
//!
//! This proc-macro intentionally uses absolute paths (e.g., `::std::option::Option`)
//! to avoid conflicts with items in the user's scope.
//!
//! See: <https://doc.rust-lang.org/reference/procedural-macros.html#procedural-macro-hygiene>

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
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Derive macro for generating options with partial struct support.
///
/// This macro generates a "partial" struct variant where all fields are wrapped
/// in `Option<T>`, along with methods for merging, resolution, and validation.
///
/// # Struct Attributes
///
/// - `#[options(partial = "PartialName")]` - Name of the generated partial struct (optional, defaults to `{StructName}Partial`)
///
/// # Field Attributes
///
/// - `#[options(required)]` - Field must be provided, resolution fails if missing
/// - `#[options(default = value)]` - Static default value if not provided
/// - `#[options(default_fn = fn_name)]` - Function to compute default from partial struct
/// - `#[options(default_doc = "text")]` - Documentation text for the default column (use with `default_fn`)
///
/// For all non-Option fields, the partial type becomes `Option<T>`.
/// `Option<T>` fields remain `Option<T>` in both structs.
#[proc_macro_derive(Options, attributes(options, arg, serde))]
pub fn derive_options(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_options_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Implement the Options derive macro.
fn derive_options_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    /// Verify macro output for all 8 combinations of (default_fn, default_value, is_option).
    #[test]
    fn expand_all_field_types() {
        let output = expand_options(
            r#"
            pub struct TestOptions {
                /// non-Option, no defaults
                pub a: String,
                /// non-Option, static default
                #[options(default = 42)]
                pub b: u32,
                /// non-Option, default_fn with doc
                #[options(default_fn = compute_c, default_doc = "computed at runtime")]
                pub c: u32,
                /// non-Option, both
                #[options(default_fn = compute_d, default = 99)]
                pub d: u32,

                /// Option, no defaults
                pub e: Option<String>,
                /// Option, static default
                #[options(default = Some("fallback".to_owned()))]
                pub f: Option<String>,
                /// Option, default_fn
                #[options(default_fn = compute_g)]
                pub g: Option<String>,
                /// Option, both
                #[options(default_fn = compute_h, default = Some("final".to_owned()))]
                pub h: Option<String>,
            }
            "#,
        );
        assert_snapshot!(output);
    }

    /// Verify macro output with custom partial name.
    #[test]
    fn expand_with_custom_partial_name() {
        let output = expand_options(
            r#"
            #[options(partial = "CustomPartial")]
            pub struct CustomOptions {
                pub value: u32,
            }
            "#,
        );
        assert_snapshot!(output);
    }

    /// Verify macro output with no commands (`from_args` returns None).
    #[test]
    fn expand_no_commands() {
        let output = expand_options(
            r"
            pub struct NoCommandOptions {
                #[options(default = 100)]
                pub limit: u32,
            }
            ",
        );
        assert_snapshot!(output);
    }

    /// Verify macro output with custom arg attributes forwarded.
    #[test]
    fn expand_with_custom_arg_attrs() {
        let output = expand_options(
            r#"
            pub struct CustomArgOptions {
                #[arg(long = "custom-name", value_name = "PATH")]
                pub file_path: String,
            }
            "#,
        );
        assert_snapshot!(output);
    }

    /// Formats generated tokens as readable Rust code using prettyplease.
    fn format_tokens(tokens: TokenStream2) -> String {
        let file = syn::parse_file(&tokens.to_string()).expect("generated code should parse");
        prettyplease::unparse(&file)
    }

    /// Expands the Options derive macro on the given struct definition.
    fn expand_options(input: &str) -> String {
        let input: DeriveInput = syn::parse_str(input).expect("input should parse");
        let tokens = derive_options_impl(input).expect("derive should succeed");
        format_tokens(tokens)
    }

    /// Extracts the struct with `#[derive(...Options...)]` from a source file and expands it.
    fn expand_options_from_file(source: &str) -> String {
        let file = syn::parse_file(source).expect("file should parse");
        for item in file.items {
            if let syn::Item::Struct(item_struct) = item {
                let has_options_derive = item_struct.attrs.iter().any(|attr| {
                    if attr.path().is_ident("derive") {
                        let mut has_options = false;
                        let _ = attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("Options") {
                                has_options = true;
                            }
                            Ok(())
                        });
                        return has_options;
                    }
                    false
                });
                if has_options_derive {
                    let input = DeriveInput {
                        attrs: item_struct.attrs,
                        vis: item_struct.vis,
                        ident: item_struct.ident,
                        generics: item_struct.generics,
                        data: syn::Data::Struct(syn::DataStruct {
                            struct_token: item_struct.struct_token,
                            fields: item_struct.fields,
                            semi_token: item_struct.semi_token,
                        }),
                    };
                    let tokens = derive_options_impl(input).expect("derive should succeed");
                    return format_tokens(tokens);
                }
            }
        }
        panic!("No struct with #[derive(Options)] found");
    }

    // ===== Snapshot tests for real options structs =====

    #[test]
    fn expand_shared_options() {
        let source = include_str!("../../core/src/options/shared_options.rs");
        assert_snapshot!(expand_options_from_file(source));
    }

    #[test]
    fn expand_batch_options() {
        let source = include_str!("../../core/src/options/batch_options.rs");
        assert_snapshot!(expand_options_from_file(source));
    }

    #[test]
    fn expand_cache_options() {
        let source = include_str!("../../core/src/options/cache_options.rs");
        assert_snapshot!(expand_options_from_file(source));
    }

    #[test]
    fn expand_copy_options() {
        let source = include_str!("../../core/src/options/copy_options.rs");
        assert_snapshot!(expand_options_from_file(source));
    }

    #[test]
    fn expand_file_options() {
        let source = include_str!("../../core/src/options/file_options.rs");
        assert_snapshot!(expand_options_from_file(source));
    }

    #[test]
    fn expand_runner_options() {
        let source = include_str!("../../core/src/options/runner_options.rs");
        assert_snapshot!(expand_options_from_file(source));
    }

    #[test]
    fn expand_spectrogram_options() {
        let source = include_str!("../../core/src/options/spectrogram_options.rs");
        assert_snapshot!(expand_options_from_file(source));
    }

    #[test]
    fn expand_target_options() {
        let source = include_str!("../../core/src/options/target_options.rs");
        assert_snapshot!(expand_options_from_file(source));
    }

    #[test]
    fn expand_upload_options() {
        let source = include_str!("../../core/src/options/upload_options.rs");
        assert_snapshot!(expand_options_from_file(source));
    }

    #[test]
    fn expand_verify_options() {
        let source = include_str!("../../core/src/options/verify_options.rs");
        assert_snapshot!(expand_options_from_file(source));
    }
}
