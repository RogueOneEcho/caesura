//! Snapshot tests for the Options and `CommandEnum` derive macros.

use insta::assert_snapshot;
use prettyplease::unparse;
use proc_macro2::TokenStream as TokenStream2;
use syn::parse_file as syn_parse_file;
use syn::parse_str as syn_parse_str;
use syn::{Data, DataStruct, DeriveInput, Item};

/// Verify macro output for all 8 combinations of (`default_fn`, `default_value`, `is_option`).
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

/// Verify macro output with no custom attributes.
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
    let file = syn_parse_file(&tokens.to_string()).expect("generated code should parse");
    unparse(&file)
}

/// Expands the Options derive macro on the given struct definition.
fn expand_options(input: &str) -> String {
    let input: DeriveInput = syn_parse_str(input).expect("input should parse");
    let tokens = super::options::derive(input).expect("derive should succeed");
    format_tokens(tokens)
}

/// Extracts the struct with `#[derive(...Options...)]` from a source file and expands it.
#[expect(
    clippy::panic,
    reason = "test helper should panic on missing derive target"
)]
fn expand_options_from_file(source: &str) -> String {
    let file = syn_parse_file(source).expect("file should parse");
    for item in file.items {
        if let Item::Struct(item_struct) = item {
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
                    data: Data::Struct(DataStruct {
                        struct_token: item_struct.struct_token,
                        fields: item_struct.fields,
                        semi_token: item_struct.semi_token,
                    }),
                };
                let tokens = super::options::derive(input).expect("derive should succeed");
                return format_tokens(tokens);
            }
        }
    }
    panic!("No struct with #[derive(Options)] found");
}

// ===== Snapshot tests for CommandEnum derive =====

#[test]
fn expand_command_enum_primary() {
    let input: DeriveInput = syn_parse_str(
        r#"
        /// App description
        #[derive(CommandEnum, Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum Command {
            /// Run batch processing
            #[options(ConfigOptions, SharedOptions)]
            Batch,

            /// Show configuration
            #[options(ConfigOptions)]
            Config,

            /// Generate docs
            Docs,

            /// Inspect files
            #[options(InspectArg)]
            Inspect,

            /// Queue commands
            #[command(subcommand_required = true, arg_required_else_help = true)]
            Queue(QueueCommand),

            /// Transcode files
            #[options(SourceArg, ConfigOptions, SharedOptions)]
            Transcode,

            /// Show version
            #[command(short_flag = 'V', long_flag = "version")]
            #[options(SoxOptions)]
            Version,
        }
        "#,
    )
    .expect("input should parse");
    let tokens = super::command::derive(input).expect("derive should succeed");
    assert_snapshot!(format_tokens(tokens));
}

#[test]
fn expand_command_enum_sub() {
    let input: DeriveInput = syn_parse_str(
        r#"
        #[derive(CommandEnum, Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[command_enum(parent = "queue")]
        pub enum QueueCommand {
            /// Add to queue
            #[options(ConfigOptions, QueueAddArgs)]
            Add,

            /// Remove from queue
            #[cli_name = "rm"]
            #[options(QueueRemoveArgs, ConfigOptions)]
            Remove,
        }
        "#,
    )
    .expect("input should parse");
    let tokens = super::command::derive(input).expect("derive should succeed");
    assert_snapshot!(format_tokens(tokens));
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
