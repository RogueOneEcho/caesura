//! `CommandEnum` derive macro implementation.

mod generate;
mod parse;

use parse::{enum_def_from_derive, extract_parent_attr};
use proc_macro2::TokenStream as TokenStream2;
use syn::DeriveInput;

/// Generate output from a `#[derive(CommandEnum)]` invocation.
pub(crate) fn derive(input: DeriveInput) -> syn::Result<TokenStream2> {
    let parent = extract_parent_attr(&input);
    let def = enum_def_from_derive(&input)?;
    if let Some(parent_cli_name) = parent {
        Ok(generate::generate_sub(&def, &parent_cli_name))
    } else {
        Ok(generate::generate_primary(&def))
    }
}
