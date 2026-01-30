// NOTE: This proc-macro intentionally uses absolute paths (e.g., `::std::option::Option`)
// to avoid conflicts with items in the user's scope. This is standard practice for proc-macros.
// See: https://doc.rust-lang.org/reference/procedural-macros.html#procedural-macro-hygiene

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, Field, Fields, Ident, Lit, Meta, Path, Type,
    parse::Parse, parse_macro_input, punctuated::Punctuated, token::Comma,
};

/// Derive macro for generating options with partial struct support.
///
/// This macro generates a "partial" struct variant where all fields are wrapped
/// in `Option<T>`, along with methods for merging, resolution, and validation.
///
/// # Struct Attributes
///
/// - `#[options(partial = "PartialName")]` - Name of the generated partial struct (optional, defaults to `{StructName}Partial`)
/// - `#[options(defaults_fn = "Self::method")]` - Function to apply calculated defaults
/// - `#[options(commands(Batch, Upload))]` - Which commands use this options struct
/// - `#[options(field_name = "field")]` - Field name in `CommandArguments` enum variants
///
/// # Field Attributes
///
/// - `#[options(default = value)]` - Default value if not provided
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
    let from_yaml_impl = generate_from_yaml_impl();
    let from_args_impl = generate_from_args_impl(&struct_opts, struct_name, &parsed_fields);
    let display_impls = generate_display_impls(struct_name, &partial_name);
    let applicable_commands_impl = generate_applicable_commands_impl(struct_name, &struct_opts);
    let applicable_commands_trait_impl =
        generate_applicable_commands_trait_impl(struct_name, &struct_opts);
    let validate_impl = generate_validate_impl(struct_name);
    let doc_metadata_impl =
        generate_doc_metadata(struct_name, &input.attrs, &parsed_fields, &struct_opts);
    Ok(quote! {
        /// Partial options struct with optional fields for CLI/YAML parsing.
        #[derive(::clap::Args, ::std::clone::Clone, ::std::fmt::Debug, ::std::default::Default, ::serde::Deserialize, ::serde::Serialize)]
        pub struct #partial_name {
            #partial_fields
        }
        impl #partial_name {
            #merge_impl
            #resolve_impl
            #from_yaml_impl
            #from_args_impl
        }
        impl crate::options::OptionsPartial for #partial_name {
            type Resolved = #struct_name;
            fn merge(&mut self, other: &Self) {
                #partial_name::merge(self, other)
            }
            fn from_yaml(yaml: &str) -> ::std::result::Result<Self, ::serde_yaml::Error> {
                #partial_name::from_yaml(yaml)
            }
            fn from_args() -> ::std::option::Option<Self> {
                #partial_name::from_args()
            }
            fn resolve(self) -> Self::Resolved {
                #partial_name::resolve(self)
            }
            fn validate(&self, errors: &mut ::std::vec::Vec<crate::options::OptionRule>) {
                #validate_impl
            }
        }
        impl #struct_name {
            #applicable_commands_impl
        }
        #display_impls
        #applicable_commands_trait_impl
        #doc_metadata_impl
    })
}

/// Returns the partial struct name, using explicit name if provided or appending "Partial" suffix.
fn get_partial_name(struct_opts: &StructOptions, struct_name: &Ident) -> Ident {
    struct_opts
        .partial_name
        .clone()
        .unwrap_or_else(|| format_ident!("{}Partial", struct_name))
}

/// Extracts named fields from a struct, returning an error for non-struct or unnamed fields.
fn extract_named_fields(input: &DeriveInput) -> syn::Result<&Punctuated<Field, Comma>> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(&fields.named),
            _ => Err(syn::Error::new_spanned(
                input,
                "Options derive only supports structs with named fields",
            )),
        },
        _ => Err(syn::Error::new_spanned(
            input,
            "Options derive only supports structs",
        )),
    }
}

struct StructOptions {
    partial_name: Option<Ident>,
    defaults_fn: Option<Path>,
    commands: Vec<Ident>,
    field_name: Option<Ident>,
    from_args_fn: Option<Path>,
}

fn parse_struct_options(attrs: &[Attribute]) -> syn::Result<StructOptions> {
    let mut opts = StructOptions {
        partial_name: None,
        defaults_fn: None,
        commands: Vec::new(),
        field_name: None,
        from_args_fn: None,
    };
    for attr in attrs {
        if !attr.path().is_ident("options") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("partial") {
                let _eq: syn::Token![=] = meta.input.parse()?;
                let lit: syn::LitStr = meta.input.parse()?;
                opts.partial_name = Some(format_ident!("{}", lit.value()));
            } else if meta.path.is_ident("defaults_fn") {
                let _eq: syn::Token![=] = meta.input.parse()?;
                let lit: syn::LitStr = meta.input.parse()?;
                opts.defaults_fn = Some(syn::parse_str(&lit.value())?);
            } else if meta.path.is_ident("from_args_fn") {
                let _eq: syn::Token![=] = meta.input.parse()?;
                let lit: syn::LitStr = meta.input.parse()?;
                opts.from_args_fn = Some(syn::parse_str(&lit.value())?);
            } else if meta.path.is_ident("commands") {
                let content;
                syn::parenthesized!(content in meta.input);
                let commands = content.parse_terminated(Ident::parse, syn::Token![,])?;
                opts.commands = commands.into_iter().collect();
            } else if meta.path.is_ident("field_name") {
                let _eq: syn::Token![=] = meta.input.parse()?;
                let lit: syn::LitStr = meta.input.parse()?;
                opts.field_name = Some(format_ident!("{}", lit.value()));
            }
            Ok(())
        })?;
    }
    Ok(opts)
}

struct ParsedField {
    ident: Ident,
    ty: Type,
    is_option: bool,
    is_bool: bool,
    default_value: Option<Expr>,
    arg_attrs: Vec<Attribute>,
    serde_attrs: Vec<Attribute>,
    doc_attrs: Vec<Attribute>,
}

fn parse_field(field: &Field) -> syn::Result<ParsedField> {
    let ident = field
        .ident
        .clone()
        .ok_or_else(|| syn::Error::new_spanned(field, "Field must have a name"))?;
    let ty = field.ty.clone();
    let is_option = is_option_type(&ty);
    let is_bool = is_bool_type(&ty);
    let mut default_value = None;
    let mut arg_attrs = Vec::new();
    let mut serde_attrs = Vec::new();
    let mut doc_attrs = Vec::new();
    for attr in &field.attrs {
        if attr.path().is_ident("options") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("default") {
                    let _eq: syn::Token![=] = meta.input.parse()?;
                    let expr: Expr = meta.input.parse()?;
                    default_value = Some(expr);
                }
                Ok(())
            })?;
        } else if attr.path().is_ident("arg") {
            arg_attrs.push(attr.clone());
        } else if attr.path().is_ident("serde") {
            serde_attrs.push(attr.clone());
        } else if attr.path().is_ident("doc") {
            doc_attrs.push(attr.clone());
        }
    }
    Ok(ParsedField {
        ident,
        ty,
        is_option,
        is_bool,
        default_value,
        arg_attrs,
        serde_attrs,
        doc_attrs,
    })
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}

fn is_bool_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "bool";
    }
    false
}

fn generate_partial_fields(fields: &[ParsedField]) -> TokenStream2 {
    let field_defs = fields.iter().map(|f| {
        let ident = &f.ident;
        let doc_attrs = &f.doc_attrs;
        let partial_type = get_partial_type(f);
        let arg_attr = generate_arg_attr(f);
        let serde_attr = generate_serde_attr(f);
        quote! {
            #(#doc_attrs)*
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

fn generate_merge_impl(fields: &[ParsedField]) -> TokenStream2 {
    let merge_stmts = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            if self.#ident.is_none() {
                self.#ident.clone_from(&other.#ident);
            }
        }
    });
    quote! {
        /// Merge another partial into self (other fills in None values)
        pub fn merge(&mut self, other: &Self) {
            #(#merge_stmts)*
        }
    }
}

fn generate_resolve_impl(
    struct_name: &Ident,
    _partial_name: &Ident,
    fields: &[ParsedField],
    struct_opts: &StructOptions,
) -> TokenStream2 {
    let resolve_stmts = fields.iter().map(generate_field_resolution);
    let field_names = fields.iter().map(|f| &f.ident);
    let apply_defaults = generate_apply_defaults_call(struct_name, struct_opts);
    quote! {
        /// Resolve partial into final options, applying defaults.
        pub fn resolve(mut self) -> #struct_name {
            #apply_defaults
            #(#resolve_stmts)*
            #struct_name {
                #(#field_names),*
            }
        }
    }
}

/// Generates resolution statement for a single field based on its attributes.
///
/// - Option fields pass through directly
/// - Fields with explicit defaults use those
/// - Other fields use `Default::default()`
fn generate_field_resolution(f: &ParsedField) -> TokenStream2 {
    let ident = &f.ident;
    if f.is_option {
        quote! {
            let #ident = self.#ident.clone();
        }
    } else if let Some(default) = &f.default_value {
        quote! {
            let #ident = self.#ident.take().unwrap_or_else(|| #default);
        }
    } else {
        quote! {
            let #ident = self.#ident.take().unwrap_or_default();
        }
    }
}

/// Generates call to `apply_calculated_defaults` if `defaults_fn` attribute is present.
fn generate_apply_defaults_call(struct_name: &Ident, struct_opts: &StructOptions) -> TokenStream2 {
    if struct_opts.defaults_fn.is_some() {
        quote! { #struct_name::apply_calculated_defaults(&mut self); }
    } else {
        quote! {}
    }
}

fn generate_from_yaml_impl() -> TokenStream2 {
    quote! {
        /// Deserialize from YAML string
        pub fn from_yaml(yaml: &str) -> ::std::result::Result<Self, ::serde_yaml::Error> {
            ::serde_yaml::from_str(yaml)
        }
    }
}

fn generate_from_args_impl(
    struct_opts: &StructOptions,
    struct_name: &Ident,
    fields: &[ParsedField],
) -> TokenStream2 {
    let reset_flags = generate_bool_flag_reset_stmts(fields);
    if struct_opts.from_args_fn.is_some() {
        return generate_from_args_with_custom_fn(struct_name, &reset_flags);
    }
    if struct_opts.commands.is_empty() {
        return generate_from_args_none();
    }
    let Some(field_name) = &struct_opts.field_name else {
        return generate_from_args_none();
    };
    let commands = &struct_opts.commands;
    let match_arms = quote! {
        #(crate::commands::CommandArguments::#commands { #field_name, .. })|* => #field_name,
    };
    quote! {
        /// Get from command line arguments
        pub fn from_args() -> ::std::option::Option<Self> {
            let mut options = match crate::commands::ArgumentsParser::get()? {
                #match_arms
                _ => return ::std::option::Option::None,
            };
            #(#reset_flags)*
            ::std::option::Option::Some(options)
        }
    }
}

/// Generates statements to reset bool flags from `Some(false)` to `None`.
///
/// Clap's `SetTrue` action sets false when the flag is absent, but we need `None`
/// to distinguish "not provided" from "explicitly set to false" during merge.
fn generate_bool_flag_reset_stmts(fields: &[ParsedField]) -> Vec<TokenStream2> {
    fields
        .iter()
        .filter(|f| f.is_bool)
        .map(|f| {
            let ident = &f.ident;
            quote! {
                if options.#ident == ::std::option::Option::Some(false) {
                    options.#ident = ::std::option::Option::None;
                }
            }
        })
        .collect()
}

fn generate_from_args_with_custom_fn(
    struct_name: &Ident,
    reset_flags: &[TokenStream2],
) -> TokenStream2 {
    quote! {
        /// Get from command line arguments
        pub fn from_args() -> ::std::option::Option<Self> {
            let mut options = #struct_name::partial_from_args()?;
            #(#reset_flags)*
            ::std::option::Option::Some(options)
        }
    }
}

fn generate_from_args_none() -> TokenStream2 {
    quote! {
        /// Get from command line arguments
        pub fn from_args() -> ::std::option::Option<Self> {
            ::std::option::Option::None
        }
    }
}

fn generate_display_impls(struct_name: &Ident, partial_name: &Ident) -> TokenStream2 {
    quote! {
        impl ::std::fmt::Display for #struct_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match ::serde_yaml::to_string(self) {
                    ::std::result::Result::Ok(yaml) => write!(f, "{}", yaml),
                    ::std::result::Result::Err(_) => write!(f, "{:?}", self),
                }
            }
        }
        impl ::std::fmt::Display for #partial_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match ::serde_yaml::to_string(self) {
                    ::std::result::Result::Ok(yaml) => write!(f, "{}", yaml),
                    ::std::result::Result::Err(_) => write!(f, "{:?}", self),
                }
            }
        }
    }
}

fn generate_applicable_commands_impl(
    _struct_name: &Ident,
    struct_opts: &StructOptions,
) -> TokenStream2 {
    let commands = &struct_opts.commands;
    if commands.is_empty() {
        quote! {
            /// Commands that use this options struct
            pub fn applicable_commands() -> &'static [crate::options::Command] {
                &[]
            }
        }
    } else {
        quote! {
            /// Commands that use this options struct
            pub fn applicable_commands() -> &'static [crate::options::Command] {
                &[#(crate::options::Command::#commands),*]
            }
        }
    }
}

fn generate_applicable_commands_trait_impl(
    struct_name: &Ident,
    struct_opts: &StructOptions,
) -> TokenStream2 {
    let commands = &struct_opts.commands;
    if commands.is_empty() {
        quote! {
            impl crate::options::ApplicableCommands for #struct_name {
                fn applicable_commands() -> &'static [crate::options::Command] {
                    &[]
                }
            }
        }
    } else {
        quote! {
            impl crate::options::ApplicableCommands for #struct_name {
                fn applicable_commands() -> &'static [crate::options::Command] {
                    &[#(crate::options::Command::#commands),*]
                }
            }
        }
    }
}

fn generate_validate_impl(struct_name: &Ident) -> TokenStream2 {
    quote! { #struct_name::validate_partial(self, errors) }
}

/// Generates the `doc_metadata` function for documentation generation.
fn generate_doc_metadata(
    struct_name: &Ident,
    struct_attrs: &[Attribute],
    fields: &[ParsedField],
    struct_opts: &StructOptions,
) -> TokenStream2 {
    let struct_name_str = struct_name.to_string();
    let struct_description = extract_doc_string(struct_attrs);
    let field_docs = fields.iter().map(|f| {
        let config_key = f.ident.to_string();
        let cli_flag = format!("--{}", config_key.replace('_', "-"));
        let field_type = type_to_display_string(&f.ty);
        let default = field_default_to_string(f);
        let description = extract_doc_string(&f.doc_attrs);

        let default_tokens = if let Some(d) = default {
            quote! { ::std::option::Option::Some(#d) }
        } else {
            quote! { ::std::option::Option::None }
        };

        quote! {
            crate::options::FieldDoc {
                config_key: #config_key,
                cli_flag: #cli_flag,
                field_type: #field_type,
                default: #default_tokens,
                description: #description,
            }
        }
    });

    let commands = struct_opts.commands.iter().map(Ident::to_string);

    quote! {
        impl crate::options::Documented for #struct_name {
            fn doc_metadata() -> &'static crate::options::OptionsDoc {
                static FIELDS: &[crate::options::FieldDoc] = &[
                    #(#field_docs),*
                ];
                static COMMANDS: &[&str] = &[#(#commands),*];
                static DOC: crate::options::OptionsDoc = crate::options::OptionsDoc {
                    name: #struct_name_str,
                    description: #struct_description,
                    fields: FIELDS,
                    commands: COMMANDS,
                };
                &DOC
            }
        }
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
        .join(" ")
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
fn field_default_to_string(f: &ParsedField) -> Option<String> {
    if let Some(default) = &f.default_value {
        Some(normalize_token_string(&quote!(#default).to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    /// Verify macro output for a struct with all field types.
    #[test]
    fn expand_all_field_types() {
        let output = expand_options(
            r#"
            #[options(commands(Batch, Transcode))]
            #[options(field_name = "opts")]
            pub struct TestOptions {
                /// A string field.
                pub name: String,

                /// A field with a default value.
                #[options(default = 42)]
                pub count: u32,

                /// A boolean flag.
                pub enabled: bool,

                /// An optional field.
                pub description: Option<String>,
            }
            "#,
        );
        assert_snapshot!(output);
    }

    /// Verify macro output with `defaults_fn` attribute.
    #[test]
    fn expand_with_defaults_fn() {
        let output = expand_options(
            r#"
            #[options(commands(Upload))]
            #[options(field_name = "opts")]
            #[options(defaults_fn = "Self::apply_calculated_defaults")]
            pub struct OptionsWithDefaults {
                pub indexer: Option<String>,
                pub url: Option<String>,
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
            #[options(commands(Verify))]
            #[options(field_name = "opts")]
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
}
