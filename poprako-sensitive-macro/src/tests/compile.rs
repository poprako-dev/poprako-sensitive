//! Runs generated code in both consumer modes without sibling package sources.

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use quote::quote;
use syn::{Data, DeriveInput, parse2};

use crate::debug::{expand, expand_format};

// Removes temporary compiler fixtures even when an assertion fails.
struct FixtureDirectory(PathBuf);

impl Drop for FixtureDirectory {
    // Cleanup failure does not obscure the original compiler/test failure.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// Expand real declarations, removing only derive helper markers afterwards.
fn declarations() -> syn::Result<String> {
    let mut source = String::new();

    for input in [
        quote! {
            pub struct Credentials<'a, T: ?Sized, const N: usize>
            where T: 'a {
                pub id: &'a str,
                #[sensitive]
                pub secret: &'a T,
                pub sequence: [u8; N],
            }
        },
        quote!(
            pub struct Pair<T>(pub u32, #[sensitive] pub T);
        ),
        quote!(
            pub struct Unit;
        ),
        quote!(
            pub struct Empty {}
        ),
        quote!(
            pub struct EmptyTuple();
        ),
        quote!(
            pub struct Raw {
                pub r#type: u32,
            }
        ),
        quote! {
            pub struct Envelope<'a> {
                pub credentials: Option<Credentials<'a, str, 2>>,
            }
        },
        quote! {
            pub struct Custom<T> {
                #[sensitive(debug = "custom_debug", display = "custom_display")]
                pub secret: T,
            }
        },
        quote! {
            pub struct EmptyPaths<T>(#[sensitive(debug = "", display = "")] pub T);
        },
        quote! {
            pub struct Failing<T>(#[sensitive(debug = "fail_format", display = "fail_format")] pub T);
        },
    ] {
        let mut declaration: DeriveInput = parse2(input.clone())?;
        let implementation = expand(input)?;
        let display = match declaration.ident.to_string().as_str() {
            "Custom" | "EmptyPaths" | "Failing" => expand_format(quote!(#declaration), true)?,
            _ => proc_macro2::TokenStream::new(),
        };

        if let Data::Struct(data) = &mut declaration.data {
            for field in &mut data.fields {
                field
                    .attrs
                    .retain(|attr| !attr.path().is_ident("sensitive"));
            }
        }

        source.push_str(&quote!(#declaration #implementation #display).to_string());
    }

    Ok(source)
}

#[test]
fn consumer_modes_redact_without_invoking_sensitive_formatters() -> Result<(), Box<dyn Error>> {
    let directory = FixtureDirectory(
        std::env::temp_dir().join(format!("poprako-sensitive-compile-{}", std::process::id())),
    );
    fs::create_dir_all(&directory.0)?;
    let source = format!("{}\n{}", declarations()?, include_str!("fixture.txt"));
    let source_path = directory.0.join("fixture.rs");
    fs::write(&source_path, source)?;

    for assertions in ["yes", "no"] {
        let executable = directory.0.join(format!("fixture-{assertions}"));
        let compiled = Command::new("rustc")
            .arg("--edition=2024")
            .arg("--test")
            .arg("-Dwarnings")
            .arg("-C")
            .arg(format!("debug-assertions={assertions}"))
            .arg(&source_path)
            .arg("-o")
            .arg(&executable)
            .output()?;
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let tested = Command::new(executable).output()?;
        assert!(
            tested.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&tested.stdout),
            String::from_utf8_lossy(&tested.stderr)
        );
    }

    Ok(())
}

#[test]
fn custom_callbacks_use_field_types_and_normal_borrow_coercions() -> Result<(), Box<dyn Error>> {
    let directory = FixtureDirectory(std::env::temp_dir().join(format!(
        "poprako-sensitive-signatures-{}",
        std::process::id()
    )));
    fs::create_dir_all(&directory.0)?;

    for (field_type, argument_type, accepted) in [
        (quote!(Email), quote!(&Email), true),
        (quote!(Email), quote!(&str), false),
        (quote!(String), quote!(&str), true),
        (quote!(Vec<u8>), quote!(&[u8]), true),
    ] {
        for display in [false, true] {
            let input = quote! {
                pub struct Invalid {
                    #[sensitive(debug = "mask", display = "mask")]
                    pub secret: #field_type,
                }
            };
            let implementation = expand_format(input, display)?;
            let source = quote! {
                pub struct Email { pub local: String, pub domain: String }
                impl ::core::fmt::Display for Email {
                    fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        write!(formatter, "{}@{}", self.local, self.domain)
                    }
                }
                pub struct Invalid { pub secret: #field_type }
                pub fn mask(_: #argument_type, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    formatter.write_str("masked")
                }
                #implementation
            };
            let source_path = directory.0.join("invalid.rs");
            fs::write(&source_path, source.to_string())?;
            let compiled = Command::new("rustc")
                .arg("--edition=2024")
                .arg("--crate-type=lib")
                .arg("--emit=metadata")
                .arg("-Dwarnings")
                .arg("-C")
                .arg("debug-assertions=no")
                .arg(&source_path)
                .arg("-o")
                .arg(directory.0.join("invalid.rmeta"))
                .output()?;
            let diagnostics = String::from_utf8_lossy(&compiled.stderr);
            assert_eq!(
                compiled.status.success(),
                accepted,
                "callback argument {argument_type} for field {field_type} (display={display}): {diagnostics}",
            );
            if !accepted {
                assert!(diagnostics.contains("E0308"), "{diagnostics}");
            }
        }
    }
    Ok(())
}
