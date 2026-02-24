//! Procedural macros for caesura.
//!
//! This crate provides:
//! - `#[derive(Options)]` — generates "partial" struct variants with `Option<T>` fields,
//!   plus trait implementations for merging, resolution, and validation.
//! - `#[derive(CommandEnum)]` — generates clap CLI infrastructure from command enums.
//!
//! # Hygiene
//!
//! This proc-macro intentionally uses absolute paths (e.g., `::std::option::Option`)
//! to avoid conflicts with items in the user's scope.
//!
//! See: <https://doc.rust-lang.org/reference/procedural-macros.html#procedural-macro-hygiene>

mod command;
mod options;

use proc_macro::TokenStream;
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
    match options::derive(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive macro for generating CLI infrastructure from command enums.
///
/// For the primary enum (e.g., `Command`), generates:
/// - `CommandArgs` enum with `#[derive(Subcommand)]`
/// - `Cli` struct with `#[derive(Parser)]`
/// - `impl CommandEnumContract` with `all()`, `cli_name()`, `doc_name()`, `about()`,
///   `options_names()`
/// - `impl Command` with `is_queue()`, `resolve()`
/// - `impl Display`
///
/// For sub-command enums (annotated with `#[command_enum(parent = "...")]`), generates:
/// - `{Name}Args` enum with `#[derive(Subcommand)]`
/// - `impl CommandEnumContract` with `all()`, `cli_name()`, `doc_name()`, `about()`,
///   `options_names()`
/// - `impl {Name}` with `resolve_sub()`
/// - `impl Display`
#[proc_macro_derive(
    CommandEnum,
    attributes(options, positional, cli_name, command, command_enum)
)]
pub fn derive_command_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match command::derive(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[cfg(test)]
mod tests;
