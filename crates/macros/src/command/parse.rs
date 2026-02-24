//! Parsing logic for command enum variants.

use quote::format_ident;
use syn::parse::ParseStream;
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, Ident, Lit, Meta, Token, Variant,
    punctuated::Punctuated,
};

/// A parsed enum definition with attributes, name, and variants.
pub struct EnumDef {
    /// Attributes on the enum (doc comments, etc.)
    pub attrs: Vec<Attribute>,
    /// Name of the enum.
    pub name: Ident,
    /// Parsed variants.
    pub variants: Vec<ParsedVariant>,
}

/// A parsed variant of a command enum.
pub struct ParsedVariant {
    /// Doc comment attributes (for forwarding).
    pub doc_attrs: Vec<Attribute>,
    /// Option type names from `#[options(Foo, Bar)]`.
    pub options: Vec<Ident>,
    /// CLI name override from `#[cli_name = "name"]`.
    pub cli_name: Option<String>,
    /// Raw `#[command(...)]` attributes to forward to clap.
    pub command_attrs: Vec<Attribute>,
    /// Variant name.
    pub name: Ident,
    /// If this variant wraps a sub-enum (e.g., `Queue(QueueCommand)`).
    pub sub_enum: Option<Ident>,
}

/// Extract `#[command_enum(parent = "...")]` from the enum attributes.
pub fn extract_parent_attr(input: &DeriveInput) -> Option<String> {
    for attr in &input.attrs {
        if !attr.path().is_ident("command_enum") {
            continue;
        }
        let mut parent = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("parent") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                parent = Some(lit.value());
            }
            Ok(())
        });
        if parent.is_some() {
            return parent;
        }
    }
    None
}

/// Build an [`EnumDef`] from a [`DeriveInput`].
pub fn enum_def_from_derive(input: &DeriveInput) -> syn::Result<EnumDef> {
    let Data::Enum(data_enum) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "CommandEnum can only be derived for enums",
        ));
    };
    let attrs: Vec<Attribute> = input
        .attrs
        .iter()
        .filter(|a| !a.path().is_ident("derive") && !a.path().is_ident("command_enum"))
        .cloned()
        .collect();
    let variants = data_enum
        .variants
        .iter()
        .cloned()
        .map(parse_variant)
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(EnumDef {
        attrs,
        name: input.ident.clone(),
        variants,
    })
}

/// Parse a single variant, extracting custom attributes.
fn parse_variant(variant: Variant) -> syn::Result<ParsedVariant> {
    let name = variant.ident;
    let mut doc_attrs = Vec::new();
    let mut options = Vec::new();
    let mut cli_name = None;
    let mut command_attrs = Vec::new();
    let mut sub_enum = None;
    if let syn::Fields::Unnamed(fields) = &variant.fields
        && fields.unnamed.len() == 1
    {
        let field = fields.unnamed.first().expect("just checked len");
        if let syn::Type::Path(type_path) = &field.ty
            && let Some(segment) = type_path.path.segments.last()
        {
            sub_enum = Some(segment.ident.clone());
        }
    }
    for attr in variant.attrs {
        if attr.path().is_ident("doc") {
            doc_attrs.push(attr);
        } else if attr.path().is_ident("options") {
            parse_ident_list(&attr, &mut options)?;
        } else if attr.path().is_ident("cli_name") {
            if let Meta::NameValue(nv) = &attr.meta
                && let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = &nv.value
            {
                cli_name = Some(s.value());
            }
        } else if attr.path().is_ident("command") {
            command_attrs.push(attr);
        }
    }
    Ok(ParsedVariant {
        doc_attrs,
        options,
        cli_name,
        command_attrs,
        name,
        sub_enum,
    })
}

/// Parse a parenthesized, comma-separated list of idents from an attribute.
fn parse_ident_list(attr: &Attribute, out: &mut Vec<Ident>) -> syn::Result<()> {
    attr.parse_args_with(|input: ParseStream| {
        let punctuated: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(input)?;
        out.extend(punctuated);
        Ok(())
    })
}

impl ParsedVariant {
    /// Effective CLI name: `#[cli_name]` override, or lowercased variant name.
    pub fn effective_cli_name(&self) -> String {
        self.cli_name
            .clone()
            .unwrap_or_else(|| self.name.to_string().to_lowercase())
    }

    /// Convert a type name like `ConfigOptions` to a `snake_case` field name.
    pub fn field_name_for(type_name: &Ident) -> Ident {
        let s = type_name.to_string();
        let snake = to_snake_case(&s);
        format_ident!("{}", snake)
    }

    /// Convert a type name like `ConfigOptions` to `ConfigOptionsPartial`.
    pub fn partial_type_for(type_name: &Ident) -> Ident {
        format_ident!("{}Partial", type_name)
    }
}

/// Convert a `PascalCase` string to `snake_case`.
///
/// Only handles simple `PascalCase` (one uppercase letter per word boundary).
/// Consecutive uppercase letters are treated individually, e.g.
/// `HTMLParser` becomes `h_t_m_l_parser`, not `html_parser`.
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().expect("char should lowercase"));
        } else {
            result.push(ch);
        }
    }
    result
}

/// Extract doc comment text from attributes.
pub fn extract_doc_string(attrs: &[Attribute]) -> String {
    attrs
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
        .join("\n")
}
