//! Code generation for the `CommandEnum` derive macro.

use super::parse::{EnumDef, ParsedVariant, extract_doc_string};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

// ===== Shared helpers =====

/// Generate the flattened fields for a variant's options.
fn generate_variant_fields(v: &ParsedVariant) -> TokenStream2 {
    let fields: Vec<TokenStream2> = v
        .options
        .iter()
        .map(|opt| {
            let field_name = ParsedVariant::field_name_for(opt);
            let partial_type = ParsedVariant::partial_type_for(opt);
            quote! {
                #[command(flatten)]
                #field_name: #partial_type
            }
        })
        .collect();
    quote! { #(#fields),* }
}

// ===== Sub-command enum (e.g., QueueCommand) =====

/// Generate all output for a sub-command enum.
pub fn generate_sub(def: &EnumDef, parent_cli_name: &str) -> TokenStream2 {
    let args_enum = generate_sub_args_enum(def);
    let sub_impl = generate_sub_impl(def, parent_cli_name);
    let display_impl = generate_sub_display(def);
    quote! {
        #args_enum
        #sub_impl
        #display_impl
    }
}

fn generate_sub_args_enum(def: &EnumDef) -> TokenStream2 {
    let args_name = format_ident!("{}Args", def.name);
    let variants: Vec<TokenStream2> = def
        .variants
        .iter()
        .map(|v| {
            let doc_attrs = &v.doc_attrs;
            let vname = &v.name;
            let command_attrs = &v.command_attrs;
            let fields = generate_variant_fields(v);
            let cli_name_attr = v
                .cli_name
                .as_ref()
                .map(|name| quote! { #[command(name = #name)] });
            quote! {
                #(#doc_attrs)*
                #(#command_attrs)*
                #cli_name_attr
                #vname {
                    #fields
                }
            }
        })
        .collect();
    quote! {
        #[derive(::clap::Subcommand, ::std::clone::Clone, ::std::fmt::Debug)]
        pub enum #args_name {
            #(#variants),*
        }
    }
}

fn generate_sub_resolve_arms(def: &EnumDef) -> Vec<TokenStream2> {
    let name = &def.name;
    def.variants
        .iter()
        .map(|v| {
            let vname = &v.name;
            let cli = v.effective_cli_name();
            quote! {
                #cli => ::std::option::Option::Some((#name::#vname, matches.clone())),
            }
        })
        .collect()
}

fn generate_sub_impl(def: &EnumDef, parent_cli_name: &str) -> TokenStream2 {
    let name = &def.name;
    let all_items: Vec<TokenStream2> = def
        .variants
        .iter()
        .map(|v| {
            let vname = &v.name;
            quote! { #name::#vname }
        })
        .collect();
    let doc_name_arms: Vec<TokenStream2> = def
        .variants
        .iter()
        .map(|v| {
            let vname = &v.name;
            let doc = format!("{} {}", parent_cli_name, v.effective_cli_name());
            quote! { #name::#vname => #doc, }
        })
        .collect();
    let options_arms: Vec<TokenStream2> = def
        .variants
        .iter()
        .map(|v| {
            let vname = &v.name;
            let names: Vec<String> = v.options.iter().map(ToString::to_string).collect();
            quote! { #name::#vname => &[#(#names),*], }
        })
        .collect();
    let resolve_arms = generate_sub_resolve_arms(def);
    quote! {
        impl crate::commands::CommandEnumContract for #name {
            /// All sub-command variants.
            fn all() -> &'static [#name] {
                &[#(#all_items),*]
            }
            /// Display name for documentation (includes parent command name).
            fn doc_name(&self) -> &'static str {
                match self {
                    #(#doc_name_arms)*
                }
            }
            /// Option type names used by this sub-command.
            fn options_names(&self) -> &'static [&'static str] {
                match self {
                    #(#options_arms)*
                }
            }
        }
        impl #name {
            /// Resolve a sub-command from a name and [`::clap::ArgMatches`].
            #[must_use]
            pub fn resolve_sub(
                name: &str,
                matches: &::clap::ArgMatches,
            ) -> ::std::option::Option<(#name, ::clap::ArgMatches)> {
                match name {
                    #(#resolve_arms)*
                    _ => ::std::option::Option::None,
                }
            }
        }
    }
}

fn generate_sub_display(def: &EnumDef) -> TokenStream2 {
    let name = &def.name;
    let arms: Vec<TokenStream2> = def
        .variants
        .iter()
        .map(|v| {
            let vname = &v.name;
            let cli = v.effective_cli_name();
            quote! { #name::#vname => f.write_str(#cli), }
        })
        .collect();
    quote! {
        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    #(#arms)*
                }
            }
        }
    }
}

// ===== Primary enum (e.g., Command) =====

/// Generate all output for a primary command enum.
pub fn generate_primary(def: &EnumDef) -> TokenStream2 {
    let args_enum = generate_primary_args_enum(def);
    let cli_struct = generate_cli_struct(def);
    let command_impl = generate_primary_impl(def);
    let display_impl = generate_primary_display(def);
    quote! {
        #args_enum
        #cli_struct
        #command_impl
        #display_impl
    }
}

fn generate_primary_args_enum(def: &EnumDef) -> TokenStream2 {
    let variants: Vec<TokenStream2> = def
        .variants
        .iter()
        .map(|v| {
            let doc_attrs = &v.doc_attrs;
            let vname = &v.name;
            let command_attrs = &v.command_attrs;
            if let Some(sub) = &v.sub_enum {
                let sub_args = format_ident!("{}Args", sub);
                let cmd_attrs = if command_attrs.is_empty() {
                    quote! { #[command(subcommand_required = true, arg_required_else_help = true)] }
                } else {
                    quote! { #(#command_attrs)* }
                };
                quote! {
                    #(#doc_attrs)*
                    #cmd_attrs
                    #vname {
                        #[command(subcommand)]
                        command: #sub_args,
                    }
                }
            } else {
                let fields = generate_variant_fields(v);
                let cli_name_attr = v
                    .cli_name
                    .as_ref()
                    .map(|name| quote! { #[command(name = #name)] });
                quote! {
                    #(#doc_attrs)*
                    #(#command_attrs)*
                    #cli_name_attr
                    #vname {
                        #fields
                    }
                }
            }
        })
        .collect();
    quote! {
        #[derive(::clap::Subcommand, ::std::clone::Clone, ::std::fmt::Debug)]
        pub enum CommandArgs {
            #(#variants),*
        }
    }
}

fn generate_cli_struct(def: &EnumDef) -> TokenStream2 {
    let doc_attrs: Vec<_> = def
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .cloned()
        .collect();
    let about_text = extract_doc_string(&doc_attrs);
    quote! {
        #[derive(::clap::Parser, ::std::clone::Clone, ::std::fmt::Debug)]
        #[command(about = #about_text)]
        pub struct Cli {
            /// The command to run.
            #[command(subcommand)]
            pub command: Option<CommandArgs>,
        }
    }
}

fn generate_primary_impl(def: &EnumDef) -> TokenStream2 {
    let name = &def.name;
    let all_fn = generate_primary_all(def);
    let doc_name_fn = generate_delegating_method(def, "doc_name", quote! { &'static str }, |v| {
        let cli = v.effective_cli_name();
        quote! { #cli }
    });
    let options_names_fn = generate_delegating_method(
        def,
        "options_names",
        quote! { &'static [&'static str] },
        |v| {
            let names: Vec<String> = v.options.iter().map(ToString::to_string).collect();
            quote! { &[#(#names),*] }
        },
    );
    let resolve_fn = generate_primary_resolve(def);
    quote! {
        impl crate::commands::CommandEnumContract for #name {
            #all_fn
            #doc_name_fn
            #options_names_fn
        }
        impl #name {
            #resolve_fn
        }
    }
}

/// Generate a method that delegates to sub-enum for sub-command variants.
fn generate_delegating_method(
    def: &EnumDef,
    method_name: &str,
    return_type: TokenStream2,
    value_fn: impl Fn(&ParsedVariant) -> TokenStream2,
) -> TokenStream2 {
    let name = &def.name;
    let method = format_ident!("{}", method_name);
    let arms: Vec<TokenStream2> = def
        .variants
        .iter()
        .map(|v| {
            let vname = &v.name;
            if v.sub_enum.is_some() {
                quote! { #name::#vname(sub) => sub.#method(), }
            } else {
                let value = value_fn(v);
                quote! { #name::#vname => #value, }
            }
        })
        .collect();
    quote! {
        fn #method(&self) -> #return_type {
            match self {
                #(#arms)*
            }
        }
    }
}

fn generate_primary_all(def: &EnumDef) -> TokenStream2 {
    let name = &def.name;
    let has_sub = def.variants.iter().any(|v| v.sub_enum.is_some());
    if !has_sub {
        let items: Vec<TokenStream2> = def
            .variants
            .iter()
            .map(|v| {
                let vname = &v.name;
                quote! { #name::#vname }
            })
            .collect();
        return quote! {
            /// All command variants.
            fn all() -> &'static [#name] {
                &[#(#items),*]
            }
        };
    }
    let stmts: Vec<TokenStream2> = def
        .variants
        .iter()
        .map(|v| {
            let vname = &v.name;
            if let Some(sub) = &v.sub_enum {
                quote! {
                    for sub in #sub::all() {
                        v.push(#name::#vname(*sub));
                    }
                }
            } else {
                quote! { v.push(#name::#vname); }
            }
        })
        .collect();
    quote! {
        /// All command variants, with sub-commands expanded.
        fn all() -> &'static [#name] {
            static ALL: ::std::sync::OnceLock<::std::vec::Vec<#name>> =
                ::std::sync::OnceLock::new();
            ALL.get_or_init(|| {
                let mut v = ::std::vec::Vec::new();
                #(#stmts)*
                v
            })
        }
    }
}

fn generate_primary_resolve(def: &EnumDef) -> TokenStream2 {
    let name = &def.name;
    let arms: Vec<TokenStream2> = def
        .variants
        .iter()
        .map(|v| {
            let vname = &v.name;
            let cli = v.effective_cli_name();
            if let Some(sub) = &v.sub_enum {
                quote! {
                    #cli => {
                        let (sub_name, sub_sub_matches) = sub_matches.subcommand()?;
                        #sub::resolve_sub(sub_name, sub_sub_matches)
                            .map(|(sub, m)| (#name::#vname(sub), m))
                    }
                }
            } else {
                quote! {
                    #cli => ::std::option::Option::Some((#name::#vname, sub_matches.clone())),
                }
            }
        })
        .collect();
    quote! {
        /// Resolve a [`Command`] and subcommand-level [`::clap::ArgMatches`] from top-level matches.
        #[must_use]
        pub fn resolve(
            matches: &::clap::ArgMatches,
        ) -> ::std::option::Option<(#name, ::clap::ArgMatches)> {
            let (name, sub_matches) = matches.subcommand()?;
            match name {
                #(#arms)*
                _ => ::std::option::Option::None,
            }
        }
    }
}

fn generate_primary_display(def: &EnumDef) -> TokenStream2 {
    let name = &def.name;
    quote! {
        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.doc_name())
            }
        }
    }
}
