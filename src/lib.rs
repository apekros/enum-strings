//! `#[derive(EnumStrings)]`: a `const fn` per named string on a unit enum,
//! where every variant must supply every string or the derive fails.
//!
//! ```
//! use enum_strings::EnumStrings;
//!
//! #[derive(Clone, Copy, EnumStrings)]
//! enum Route {
//!     #[strings(path = "/", label = "Home")]
//!     Home,
//!     #[strings(path = "/docs", label = "Docs")]
//!     Docs,
//! }
//!
//! assert_eq!(Route::Docs.path(), "/docs");
//! assert_eq!(Route::Home.label(), "Home");
//! ```
//!
//! The first variant decides which strings exist. A variant that leaves one
//! out, adds one, or repeats one is a compile error on that variant, which is
//! the check a `match` gives you and a runtime property lookup does not.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Error, Fields, Ident, LitStr, Result, Variant};

/// See the [crate documentation](crate).
#[proc_macro_derive(EnumStrings, attributes(strings))]
pub fn derive_enum_strings(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    syn::parse2(TokenStream::from(input))
        .and_then(|input| expand(&input))
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// The strings of one variant, in attribute order.
struct Strings {
    /// The variant they belong to.
    variant: Ident,
    /// Each `key = "value"`.
    values: Vec<(Ident, LitStr)>,
}

/// Parses every `#[strings(key = "value")]` on a variant.
fn parse_variant(variant: &Variant) -> Result<Strings> {
    if !matches!(variant.fields, Fields::Unit) {
        return Err(Error::new_spanned(
            variant,
            "EnumStrings only supports unit variants",
        ));
    }
    let mut values: Vec<(Ident, LitStr)> = Vec::new();
    for attr in variant
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("strings"))
    {
        attr.parse_nested_meta(|meta| {
            let key = meta.path.require_ident()?.clone();
            if values.iter().any(|(seen, _)| *seen == key) {
                return Err(meta.error(format!("`{key}` given twice")));
            }
            let value: LitStr = meta.value()?.parse()?;
            values.push((key, value));
            Ok(())
        })?;
    }
    if values.is_empty() {
        return Err(Error::new_spanned(
            variant,
            "missing `#[strings(...)]`, every variant must supply every string",
        ));
    }
    Ok(Strings {
        variant: variant.ident.clone(),
        values,
    })
}

/// Checks a variant against the strings the first variant declared.
fn check_keys(strings: &Strings, keys: &[Ident]) -> Result<()> {
    let expected = keys
        .iter()
        .map(|key| format!("`{key}`"))
        .collect::<Vec<_>>()
        .join(", ");
    for (key, _) in &strings.values {
        if !keys.contains(key) {
            return Err(Error::new(
                key.span(),
                format!("unknown string `{key}`, the first variant declares {expected}"),
            ));
        }
    }
    for key in keys {
        if !strings.values.iter().any(|(seen, _)| seen == key) {
            return Err(Error::new(
                strings.variant.span(),
                format!("variant `{}` is missing `{key}`", strings.variant),
            ));
        }
    }
    Ok(())
}

/// Generates the `impl` with one `const fn` per string.
fn expand(input: &DeriveInput) -> Result<TokenStream> {
    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(input, "EnumStrings only supports enums"));
    };
    let variants = data
        .variants
        .iter()
        .map(parse_variant)
        .collect::<Result<Vec<_>>>()?;
    let keys: Vec<Ident> = variants
        .first()
        .map(|first| first.values.iter().map(|(key, _)| key.clone()).collect())
        .unwrap_or_default();
    for strings in &variants {
        check_keys(strings, &keys)?;
    }

    let name = &input.ident;
    let vis = &input.vis;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let methods = keys.iter().map(|key| {
        let doc = format!("The `{key}` of this variant.");
        let method = format_ident!("{key}");
        let arms = variants.iter().map(|strings| {
            let variant = &strings.variant;
            // `check_keys` guarantees the key is present on every variant.
            let value = strings
                .values
                .iter()
                .find(|(seen, _)| seen == key)
                .map(|(_, value)| value);
            quote! { Self::#variant => #value }
        });
        quote! {
            #[doc = #doc]
            #[must_use]
            #vis const fn #method(self) -> &'static str {
                match self {
                    #(#arms),*
                }
            }
        }
    });
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #name #ty_generics #where_clause {
            #(#methods)*
        }
    })
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    /// The error message the derive reports for `input`, or `None` when it
    /// expands.
    fn error(input: &DeriveInput) -> Option<String> {
        expand(input).err().map(|error| error.to_string())
    }

    #[test]
    fn generates_a_const_fn_per_string() -> anyhow::Result<()> {
        let input: DeriveInput = parse_quote! {
            pub enum Route {
                #[strings(path = "/", label = "Home")]
                Home,
                #[strings(label = "Docs", path = "/docs")]
                Docs,
            }
        };
        let expanded = expand(&input)?.to_string();
        assert!(
            expanded.contains("pub const fn path (self) -> & 'static str"),
            "{expanded}"
        );
        assert!(
            expanded.contains("pub const fn label (self) -> & 'static str"),
            "{expanded}"
        );
        assert!(expanded.contains("Self :: Docs => \"/docs\""), "{expanded}");
        assert!(expanded.contains("Self :: Docs => \"Docs\""), "{expanded}");
        Ok(())
    }

    #[test]
    fn every_variant_must_supply_every_string() {
        let input: DeriveInput = parse_quote! {
            enum Route {
                #[strings(path = "/", label = "Home")]
                Home,
                #[strings(path = "/docs")]
                Docs,
            }
        };
        assert_eq!(
            error(&input).as_deref(),
            Some("variant `Docs` is missing `label`")
        );

        let input: DeriveInput = parse_quote! {
            enum Route {
                #[strings(path = "/", label = "Home")]
                Home,
                Docs,
            }
        };
        assert_eq!(
            error(&input).as_deref(),
            Some("missing `#[strings(...)]`, every variant must supply every string")
        );
    }

    #[test]
    fn strings_must_match_the_first_variant() {
        let input: DeriveInput = parse_quote! {
            enum Route {
                #[strings(path = "/", label = "Home")]
                Home,
                #[strings(path = "/docs", lable = "Docs")]
                Docs,
            }
        };
        assert_eq!(
            error(&input).as_deref(),
            Some("unknown string `lable`, the first variant declares `path`, `label`")
        );

        let input: DeriveInput = parse_quote! {
            enum Route {
                #[strings(path = "/", path = "/again")]
                Home,
            }
        };
        assert_eq!(error(&input).as_deref(), Some("`path` given twice"));
    }

    #[test]
    fn only_unit_variants_of_enums() {
        let input: DeriveInput = parse_quote! {
            enum Route {
                #[strings(path = "/")]
                Home(u8),
            }
        };
        assert_eq!(
            error(&input).as_deref(),
            Some("EnumStrings only supports unit variants")
        );

        let input: DeriveInput = parse_quote! {
            struct Route;
        };
        assert_eq!(
            error(&input).as_deref(),
            Some("EnumStrings only supports enums")
        );
    }
}
