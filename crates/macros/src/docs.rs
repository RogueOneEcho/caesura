//! Documentation metadata generation for the Options derive macro.

use crate::parse::ParsedField;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Expr, ExprLit, Ident, Lit, Meta, Type};

/// Generates the `doc_metadata` function for documentation generation.
pub fn generate_doc_metadata(
    struct_name: &Ident,
    struct_attrs: &[Attribute],
    fields: &[ParsedField],
) -> TokenStream2 {
    let struct_name_str = struct_name.to_string();
    let struct_description = extract_doc_string(struct_attrs);
    let field_docs = fields.iter().map(|f| {
        let config_key = f.ident.to_string();
        let cli_flag = format!("--{}", config_key.replace('_', "-"));
        let field_type = type_to_display_string(&f.ty);
        let description = extract_doc_string(&f.doc_attrs);
        let default_value_expr = generate_default_value_expr(f);
        let default_doc_expr = generate_default_doc_expr(f);
        quote! {
            crate::options::FieldDoc {
                config_key: #config_key,
                cli_flag: #cli_flag,
                field_type: #field_type,
                default_value: #default_value_expr,
                default_doc: #default_doc_expr,
                description: #description,
            }
        }
    });

    quote! {
        impl crate::options::Documented for #struct_name {
            fn doc_metadata() -> &'static crate::options::OptionsDoc {
                static DOC: ::std::sync::LazyLock<crate::options::OptionsDoc> =
                    ::std::sync::LazyLock::new(|| crate::options::OptionsDoc {
                        name: #struct_name_str,
                        description: #struct_description,
                        fields: ::std::vec![#(#field_docs),*],
                    });
                &DOC
            }
        }
    }
}

/// Generates the expression for the serialized default value.
fn generate_default_value_expr(f: &ParsedField) -> TokenStream2 {
    if let Some(default) = &f.default_value {
        quote! { ::std::option::Option::Some(::serde_json::to_string(&(#default)).unwrap()) }
    } else if f.is_bool {
        quote! { ::std::option::Option::Some("false".to_owned()) }
    } else if !f.is_option {
        // Non-Option fields use Default::default()
        let ty = &f.ty;
        quote! { ::std::option::Option::Some(::serde_json::to_string(&<#ty>::default()).unwrap()) }
    } else {
        quote! { ::std::option::Option::None }
    }
}

/// Generates the expression for the default documentation string.
fn generate_default_doc_expr(f: &ParsedField) -> TokenStream2 {
    if let Some(doc) = &f.default_doc {
        quote! { ::std::option::Option::Some(#doc) }
    } else {
        quote! { ::std::option::Option::None }
    }
}

/// Extracts doc comment text from doc attributes, joining multiple lines.
fn extract_doc_string(doc_attrs: &[Attribute]) -> String {
    doc_attrs
        .iter()
        .filter_map(|attr| {
            if let Meta::NameValue(nv) = &attr.meta
                && nv.path.is_ident("doc")
                && let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = &nv.value
            {
                return Some(s.value());
            }
            None
        })
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("<br>")
}

/// Converts a type to a display string for documentation.
fn type_to_display_string(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let ident = segment.ident.to_string();
                // Handle Option<T> specially
                if ident == "Option"
                    && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                    && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                {
                    return format!("Option<{}>", type_to_display_string(inner));
                }
                // Handle Vec<T> specially
                if ident == "Vec"
                    && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                    && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                {
                    return format!("Vec<{}>", type_to_display_string(inner));
                }
                return ident;
            }
            quote!(#ty).to_string()
        }
        _ => quote!(#ty).to_string(),
    }
}

/// Converts a field's default value to a string for documentation.
pub fn field_default_to_string(f: &ParsedField) -> Option<String> {
    if let Some(default) = &f.default_value {
        Some(normalize_token_string(&quote!(#default).to_string()))
    } else if let Some(doc) = &f.default_doc {
        Some(doc.clone())
    } else if f.is_bool {
        Some("false".to_owned())
    } else {
        None
    }
}

/// Normalizes token stringification by removing excess whitespace around punctuation.
fn normalize_token_string(s: &str) -> String {
    s.replace(" :: ", "::")
        .replace(" !", "!")
        .replace("! ", "!")
        .replace(" (", "(")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(" [", "[")
        .replace("[ ", "[")
        .replace(" ]", "]")
}
