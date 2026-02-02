//! Code generation for the Options derive macro.

use crate::docs::field_default_to_string;
use crate::parse::{ParsedField, StructOptions};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

/// Generate field definitions for the partial struct.
pub fn generate_partial_fields(fields: &[ParsedField]) -> TokenStream2 {
    let field_defs = fields.iter().map(|f| {
        let ident = &f.ident;
        let doc_attrs = &f.doc_attrs;
        let default_doc_attr = field_default_to_string(f).map(|d| {
            let doc = format!(" Default: `{d}`");
            quote! { #[doc = ""] #[doc = #doc] }
        });
        let partial_type = get_partial_type(f);
        let arg_attr = generate_arg_attr(f);
        let serde_attr = generate_serde_attr(f);
        quote! {
            #(#doc_attrs)*
            #default_doc_attr
            #arg_attr
            #serde_attr
            pub #ident: #partial_type,
        }
    });
    quote! { #(#field_defs)* }
}

/// Returns `Option<T>` for non-Option fields, or the original type for Option fields.
fn get_partial_type(f: &ParsedField) -> TokenStream2 {
    if f.is_option {
        let ty = &f.ty;
        quote! { #ty }
    } else {
        let ty = &f.ty;
        quote! { Option<#ty> }
    }
}

/// Generates clap `#[arg(...)]` attribute for a field.
///
/// Bool fields use `SetTrue` action so the flag sets true when present.
/// Other fields use forwarded attributes or default `long` argument.
fn generate_arg_attr(f: &ParsedField) -> TokenStream2 {
    let ident_str = f.ident.to_string().replace('_', "-");
    if f.is_bool {
        quote! {
            #[arg(long = #ident_str, default_value = None, action = ::clap::ArgAction::SetTrue)]
        }
    } else if !f.arg_attrs.is_empty() {
        let attrs = &f.arg_attrs;
        quote! { #(#attrs)* }
    } else {
        quote! {
            #[arg(long = #ident_str)]
        }
    }
}

/// Generates serde attribute, forwarding user attributes or using default `skip_serializing_if`.
fn generate_serde_attr(f: &ParsedField) -> TokenStream2 {
    if f.serde_attrs.is_empty() {
        quote! {
            #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        }
    } else {
        let attrs = &f.serde_attrs;
        quote! { #(#attrs)* }
    }
}

/// Generate the `merge` method for [`OptionsPartialContract`].
pub fn generate_merge_impl(fields: &[ParsedField]) -> TokenStream2 {
    let merge_stmts = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            if self.#ident.is_none() {
                self.#ident = other.#ident;
            }
        }
    });
    quote! {
        fn merge(&mut self, other: Self) {
            #(#merge_stmts)*
        }
    }
}

/// Generated resolve implementation split into inherent and trait parts.
pub struct ResolveImpl {
    /// Inherent impl block with `resolve_internal` method.
    pub inherent: TokenStream2,
    /// Trait methods that delegate to `resolve_internal`.
    pub trait_methods: TokenStream2,
}

/// Generate `resolve` and `resolve_without_validation` methods for [`OptionsPartialContract`].
pub fn generate_resolve_impl(
    struct_name: &Ident,
    partial_name: &Ident,
    fields: &[ParsedField],
    _struct_opts: &StructOptions,
) -> ResolveImpl {
    let has_default_fn = fields.iter().any(|f| f.default_fn.is_some());
    let field_bindings = fields.iter().map(generate_field_binding);
    let required_checks = generate_required_checks(fields);
    let field_inits = fields.iter().map(generate_field_init);
    let self_param = if has_default_fn {
        quote! { mut self }
    } else {
        quote! { self }
    };
    let defaults_binding = if has_default_fn {
        quote! { let defaults = self.clone(); }
    } else {
        quote! {}
    };
    let inherent = quote! {
        impl #partial_name {
            fn resolve_internal(#self_param, validate: bool) -> ::std::result::Result<#struct_name, ::std::vec::Vec<crate::options::OptionRule>> {
                let mut errors = ::std::vec::Vec::new();
                #defaults_binding
                #(#field_bindings)*
                #required_checks
                let resolved = #struct_name { #(#field_inits),* };
                if validate {
                    resolved.validate(&mut errors);
                }
                if errors.is_empty() {
                    ::std::result::Result::Ok(resolved)
                } else {
                    ::std::result::Result::Err(errors)
                }
            }
        }
    };
    let trait_methods = quote! {
        fn resolve_without_validation(self) -> #struct_name {
            self.resolve_internal(false).expect("validation disabled")
        }

        fn resolve(self) -> ::std::result::Result<#struct_name, ::std::vec::Vec<crate::options::OptionRule>> {
            self.resolve_internal(true)
        }
    };
    ResolveImpl {
        inherent,
        trait_methods,
    }
}

/// Generates a let binding for a field, applying `default_fn` if present.
fn generate_field_binding(f: &ParsedField) -> TokenStream2 {
    let ident = &f.ident;
    let mut expr = quote! { self.#ident };
    if let Some(default_fn) = &f.default_fn {
        expr = quote! { #expr.or_else(|| #default_fn(&defaults)) };
    }
    quote! { let #ident = #expr; }
}

/// Generates checks for required fields, adding `NotSet` errors if missing.
fn generate_required_checks(fields: &[ParsedField]) -> TokenStream2 {
    let checks = fields.iter().filter(|f| f.is_required).map(|f| {
        let ident = &f.ident;
        let name = field_name_for_error(ident);
        quote! {
            if validate && #ident.is_none() {
                errors.push(crate::options::OptionRule::NotSet(#name.to_owned()));
            }
        }
    });
    quote! { #(#checks)* }
}

/// Converts a field identifier to a human-readable name for error messages.
fn field_name_for_error(ident: &Ident) -> String {
    let name = ident.to_string().replace('_', " ");
    name.chars()
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + &name[1..])
        .unwrap_or_default()
}

/// Generates a field initializer for struct construction.
fn generate_field_init(f: &ParsedField) -> TokenStream2 {
    let ident = &f.ident;
    if let Some(default_value) = &f.default_value {
        if f.is_option {
            quote! { #ident: #ident.or_else(|| #default_value) }
        } else {
            quote! { #ident: #ident.unwrap_or_else(|| #default_value) }
        }
    } else if f.is_option {
        quote! { #ident }
    } else {
        quote! { #ident: #ident.unwrap_or_default() }
    }
}

/// Generates the `Default` impl for the resolved struct.
pub fn generate_default_impl(struct_name: &Ident, fields: &[ParsedField]) -> TokenStream2 {
    let field_defaults = fields.iter().map(generate_field_default);
    quote! {
        impl ::std::default::Default for #struct_name {
            fn default() -> Self {
                Self {
                    #(#field_defaults),*
                }
            }
        }
    }
}

/// Generates a field initializer for the Default impl.
fn generate_field_default(f: &ParsedField) -> TokenStream2 {
    let ident = &f.ident;
    if f.is_option {
        quote! { #ident: ::std::option::Option::None }
    } else if let Some(default) = &f.default_value {
        quote! { #ident: #default }
    } else {
        quote! { #ident: ::std::default::Default::default() }
    }
}
