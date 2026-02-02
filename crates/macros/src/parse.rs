//! Parsing logic for the Options derive macro.

use quote::format_ident;
use syn::{
    Attribute, Data, DeriveInput, Expr, Field, Fields, Ident, Path, Type, punctuated::Punctuated,
    token::Comma,
};

/// Struct-level options parsed from `#[options(...)]` attributes.
pub struct StructOptions {
    /// Custom name for the generated partial struct.
    pub partial_name: Option<Ident>,
}

/// Parse struct-level `#[options(...)]` attributes.
pub fn parse_struct_options(attrs: &[Attribute]) -> syn::Result<StructOptions> {
    let mut opts = StructOptions { partial_name: None };
    for attr in attrs {
        if !attr.path().is_ident("options") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("partial") {
                let _eq: syn::Token![=] = meta.input.parse()?;
                let lit: syn::LitStr = meta.input.parse()?;
                opts.partial_name = Some(format_ident!("{}", lit.value()));
            }
            Ok(())
        })?;
    }
    Ok(opts)
}

/// Returns the partial struct name, using explicit name if provided or appending "Partial" suffix.
pub fn get_partial_name(struct_opts: &StructOptions, struct_name: &Ident) -> Ident {
    struct_opts
        .partial_name
        .clone()
        .unwrap_or_else(|| format_ident!("{}Partial", struct_name))
}

/// Extracts named fields from a struct, returning an error for non-struct or unnamed fields.
pub fn extract_named_fields(input: &DeriveInput) -> syn::Result<&Punctuated<Field, Comma>> {
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

/// Parsed representation of a struct field with all relevant attributes.
pub struct ParsedField {
    /// Field name.
    pub ident: Ident,
    /// Field type.
    pub ty: Type,
    /// Whether the field type is `Option<T>`.
    pub is_option: bool,
    /// Whether the field type is `bool`.
    pub is_bool: bool,
    /// Whether the field is marked `#[options(required)]`.
    pub is_required: bool,
    /// Static default value from `#[options(default = ...)]`.
    pub default_value: Option<Expr>,
    /// Function to compute default from `#[options(default_fn = ...)]`.
    pub default_fn: Option<Path>,
    /// Documentation for computed defaults from `#[options(default_doc = ...)]`.
    pub default_doc: Option<String>,
    /// Forwarded `#[arg(...)]` attributes.
    pub arg_attrs: Vec<Attribute>,
    /// Forwarded `#[serde(...)]` attributes.
    pub serde_attrs: Vec<Attribute>,
    /// Doc comment attributes.
    pub doc_attrs: Vec<Attribute>,
}

/// Parse a struct field into a [`ParsedField`].
pub fn parse_field(field: &Field) -> syn::Result<ParsedField> {
    let ident = field
        .ident
        .clone()
        .ok_or_else(|| syn::Error::new_spanned(field, "Field must have a name"))?;
    let ty = field.ty.clone();
    let is_option = is_option_type(&ty);
    let is_bool = is_bool_type(&ty);
    let mut is_required = false;
    let mut default_value = None;
    let mut default_fn = None;
    let mut default_doc = None;
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
                } else if meta.path.is_ident("default_fn") {
                    let _eq: syn::Token![=] = meta.input.parse()?;
                    let path: Path = meta.input.parse()?;
                    default_fn = Some(path);
                } else if meta.path.is_ident("default_doc") {
                    let _eq: syn::Token![=] = meta.input.parse()?;
                    let lit: syn::LitStr = meta.input.parse()?;
                    default_doc = Some(lit.value());
                } else if meta.path.is_ident("required") {
                    is_required = true;
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
        is_required,
        default_value,
        default_fn,
        default_doc,
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
