# enum-strings

[![crates.io](https://img.shields.io/crates/v/enum-strings.svg)](https://crates.io/crates/enum-strings)
[![docs.rs](https://docs.rs/enum-strings/badge.svg)](https://docs.rs/enum-strings)

`#[derive(EnumStrings)]` gives a unit enum one `const fn` per named string,
and refuses to compile unless every variant supplies every string.

```rust
use enum_strings::EnumStrings;

#[derive(Clone, Copy, EnumStrings)]
pub enum Route {
    #[strings(path = "/", label = "Home")]
    Home,
    #[strings(path = "/docs", label = "Docs")]
    Docs,
}

const HOME: &str = Route::Home.path();

assert_eq!(Route::Docs.path(), "/docs");
assert_eq!(Route::Docs.label(), "Docs");
```

The derive expands to:

```rust
impl Route {
    pub const fn path(self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Docs => "/docs",
        }
    }
    pub const fn label(self) -> &'static str { /* same shape */ }
}
```

## Why

Two `match` blocks do this job fine. What they don't do is keep the path and
the label next to the variant they belong to, and once a few of these string
tables pile up on one enum, the attribute form is easier to read and to
review.

Runtime property lookups (`get_str("label")` returning `Option`) put the
strings on the variant too, but a forgotten one is `None` at runtime. Here it
is a compile error on the variant that forgot it:

```text
error: variant `Docs` is missing `label`
 --> src/route.rs:8:5
  |
8 |     Docs,
  |     ^^^^
```

## Rules

- The first variant declares which strings exist. Every other variant must
  supply exactly that set: no missing, no extra, no repeats.
- Keys become method names, so a typo at a call site is an ordinary "no method
  named" error instead of a silent miss.
- Only unit variants. Generated methods take `self` by value, so the enum
  should be `Copy`.
- The methods are `pub` when the enum is, `#[must_use]`, and documented, so
  `missing_docs` stays quiet.

## License

MIT OR Apache-2.0, at your option.
