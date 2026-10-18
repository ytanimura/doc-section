#![doc = include_str!("../README.md")]

mod expand;
mod markdown;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Attaches the body of a Markdown h2 section to an item's documentation.
///
/// With no argument, processes nested `#[doc_section("file.md#section")]`
/// markers without adding documentation to the enclosing item.
#[proc_macro_attribute]
pub fn doc_section(args: TokenStream, input: TokenStream) -> TokenStream {
    expand::attribute(args.into(), input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Expands the body of a Markdown h2 section into a string literal.
///
/// Use this in `#![doc = include_doc_section!("file.md#overview")]`.
/// Register the Markdown file in the consuming crate's `build.rs` to track
/// changes. See the crate documentation for details.
#[proc_macro]
pub fn include_doc_section(input: TokenStream) -> TokenStream {
    let selector = parse_macro_input!(input as syn::LitStr);
    expand::include(&selector)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
