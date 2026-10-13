//! Generates independent Debug and Display implementations for both consumer modes.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Error, Field, Fields, Generics, Index, LitStr, Meta, Path, parse_quote,
    parse2,
};

// Optional paths are independent; empty strings select default redaction.
#[derive(Default)]
struct SensitiveOptions {
    debug: Option<Path>,
    display: Option<Path>,
}

impl SensitiveOptions {
    const fn path(&self, display: bool) -> Option<&Path> {
        if display {
            return self.display.as_ref();
        }

        self.debug.as_ref()
    }
}

// Validate field metadata and reject duplicate keys even when their values are empty.
fn sensitive_options(field: &Field) -> syn::Result<Option<SensitiveOptions>> {
    let mut options = None;

    for attribute in &field.attrs {
        if !attribute.path().is_ident("sensitive") {
            continue;
        }

        if options.is_some() {
            return Err(Error::new_spanned(
                attribute,
                "use exactly one #[sensitive] marker",
            ));
        }

        let mut parsed = SensitiveOptions::default();

        match &attribute.meta {
            Meta::Path(_) => {}
            Meta::List(_) => {
                let mut debug_seen = false;
                let mut display_seen = false;
                attribute.parse_nested_meta(|meta| {
                    let (seen, path) = match () {
                        () if meta.path.is_ident("debug") => (&mut debug_seen, &mut parsed.debug),
                        () if meta.path.is_ident("display") => {
                            (&mut display_seen, &mut parsed.display)
                        }
                        () => {
                            return Err(meta.error("expected debug or display"));
                        }
                    };

                    if *seen {
                        return Err(meta.error("duplicate formatter key"));
                    }

                    *seen = true;
                    let value: LitStr = meta.value()?.parse()?;

                    if value.value().is_empty() {
                        return Ok(());
                    }

                    *path = Some(value.parse()?);
                    Ok(())
                })?;
            }
            Meta::NameValue(_) => {
                return Err(Error::new_spanned(
                    attribute,
                    "expected #[sensitive] or #[sensitive(debug = \"path\", display = \"path\")]",
                ));
            }
        }

        options = Some(parsed);
    }

    Ok(options)
}

// Select the trait without imposing bounds on custom or hidden field values.
fn format_trait(display: bool) -> TokenStream {
    if display {
        return quote!(::core::fmt::Display);
    }

    quote!(::core::fmt::Debug)
}

// Select a field expression without touching default-redacted values.
fn field_value(
    field: &Field,
    index: usize,
    generics: &mut Generics,
    sensitive: Option<&SensitiveOptions>,
    display: bool,
) -> TokenStream {
    let value = field.ident.as_ref().map_or_else(
        || {
            let index = Index::from(index);
            quote!(&self.#index)
        },
        |name| quote!(&self.#name),
    );

    if let Some(options) = sensitive {
        let Some(path) = options.path(display) else {
            if display {
                return quote!(&SensitiveFormat(
                    |formatter: &mut ::core::fmt::Formatter<'_>| formatter.write_str("[REDACTED]")
                ));
            }

            return quote!(&"[REDACTED]");
        };
        return quote!(&SensitiveFormat(|formatter: &mut ::core::fmt::Formatter<'_>| #path(#value, formatter)));
    }

    let field_type = &field.ty;
    let format_trait = format_trait(display);
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#field_type: #format_trait));

    if display {
        return quote!(&SensitiveFormat(|formatter: &mut ::core::fmt::Formatter<'_>| #format_trait::fmt(#value, formatter)));
    }

    value
}

// Keep configuration selection in the generated consumer code.
fn assertion_cfg(reveal: bool) -> TokenStream {
    if reveal {
        return quote!(debug_assertions);
    }

    quote!(not(debug_assertions))
}

// Wrap formatting callbacks without allocating an intermediate string.
fn format_adapter(enabled: bool) -> TokenStream {
    if !enabled {
        return TokenStream::new();
    }

    quote! {
        struct SensitiveFormat<F>(F);
        impl<F> ::core::fmt::Debug for SensitiveFormat<F>
        where F: ::core::ops::Fn(&mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                (self.0)(formatter)
            }
        }
    }
}

// Generate one mode with bounds only on fields that will actually be formatted.
fn implementation(
    input: &DeriveInput,
    fields: &Fields,
    sensitive: &[Option<SensitiveOptions>],
    reveal: bool,
    display: bool,
) -> TokenStream {
    let name = &input.ident;
    let mut generics = input.generics.clone();
    let mut statements = Vec::new();

    for (index, (field, sensitive)) in fields.iter().zip(sensitive).enumerate() {
        let value = field_value(
            field,
            index,
            &mut generics,
            sensitive.as_ref().filter(|_| !reveal),
            display,
        );
        let statement = field.ident.as_ref().map_or_else(
            || quote!(builder.field(#value);),
            |name| {
                let label = name.to_string().trim_start_matches("r#").to_owned();
                quote!(builder.field(#label, #value);)
            },
        );
        statements.push(statement);
    }

    let adapter = format_adapter(sensitive.iter().any(|options| {
        display
            || options
                .as_ref()
                .filter(|_| !reveal)
                .is_some_and(|options| options.debug.is_some())
    }));
    let body = match fields {
        Fields::Named(_) => quote! {
            let mut builder = formatter.debug_struct(stringify!(#name));
            #(#statements)*
            builder.finish()
        },
        Fields::Unnamed(_) => quote! {
            let mut builder = formatter.debug_tuple(stringify!(#name));
            #(#statements)*
            builder.finish()
        },
        Fields::Unit => quote!(formatter.write_str(stringify!(#name))),
    };
    let condition = assertion_cfg(reveal);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let format_trait = format_trait(display);
    quote! {
        #[cfg(#condition)]
        impl #impl_generics #format_trait for #name #type_generics #where_clause {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #adapter
                #body
            }
        }
    }
}

/// Expands a struct declaration into Debug implementations without rewriting it.
pub fn expand(tokens: TokenStream) -> syn::Result<TokenStream> {
    expand_format(tokens, false)
}

/// Expands the selected formatting trait with independent consumer configurations.
pub fn expand_format(tokens: TokenStream, display: bool) -> syn::Result<TokenStream> {
    let input = parse2::<DeriveInput>(tokens)?;

    if let Some(attribute) = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("sensitive"))
    {
        return Err(Error::new_spanned(
            attribute,
            "#[sensitive] belongs on a struct field",
        ));
    }

    let Data::Struct(data) = &input.data else {
        return Err(Error::new_spanned(
            &input.ident,
            "sensitive formatting supports structs only",
        ));
    };
    let sensitive = data
        .fields
        .iter()
        .map(sensitive_options)
        .collect::<syn::Result<Vec<_>>>()?;
    let visible = implementation(&input, &data.fields, &sensitive, true, display);
    let redacted = implementation(&input, &data.fields, &sensitive, false, display);
    Ok(quote!(#visible #redacted))
}
