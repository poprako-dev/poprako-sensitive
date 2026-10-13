//! Compile and run both assertion modes without rebuilding the server.

mod compile;

use quote::quote;

use crate::debug::expand;

#[test]
fn invalid_markers_and_unsupported_items_are_rejected() {
    //
    for input in [
        quote!(
            struct Invalid {
                #[sensitive(always)]
                secret: String,
            }
        ),
        quote!(
            struct Invalid {
                #[sensitive = true]
                secret: String,
            }
        ),
        quote!(
            struct Invalid {
                #[sensitive]
                #[sensitive]
                secret: String,
            }
        ),
        quote!(
            #[sensitive]
            struct Invalid {
                secret: String,
            }
        ),
        quote!(
            enum Invalid {
                Value,
            }
        ),
        quote!(union Invalid { value: u32 }),
        quote!(
            struct Invalid {
                #[sensitive(debug = "", debug = "path")]
                secret: String,
            }
        ),
        quote!(
            struct Invalid {
                #[sensitive(display = "", display = "")]
                secret: String,
            }
        ),
        quote!(
            struct Invalid {
                #[sensitive(debug = 1)]
                secret: String,
            }
        ),
        quote!(
            struct Invalid {
                #[sensitive(debug = "not a path")]
                secret: String,
            }
        ),
        quote!(
            struct Invalid {
                #[sensitive(display)]
                secret: String,
            }
        ),
    ] {
        assert!(expand(input).is_err());
    }
}

#[test]
fn optional_paths_and_empty_metadata_are_accepted() {
    for marker in [
        quote!(#[sensitive()]),
        quote!(#[sensitive(debug = "")]),
        quote!(#[sensitive(display = "")]),
        quote!(#[sensitive(debug = "crate::mask")]),
        quote!(#[sensitive(display = "crate::mask::<u32>")]),
    ] {
        let input = quote!(struct Valid { #marker secret: u32 });

        assert!(expand(input).is_ok());
    }
}
