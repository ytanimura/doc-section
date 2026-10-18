use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::{collections::BTreeSet, fs, path::PathBuf};
use syn::{Attribute, Item, LitStr, Meta, spanned::Spanned, visit_mut::VisitMut};

use crate::markdown;

fn load(selector: &LitStr) -> syn::Result<(String, PathBuf)> {
    let value = selector.value();
    let (file, section) =
        split_selector(&value).map_err(|message| syn::Error::new(selector.span(), message))?;
    let source_file = selector.span().unwrap().local_file().ok_or_else(|| {
        syn::Error::new(
            selector.span(),
            "cannot locate the invoking Rust source file",
        )
    })?;
    let parent = source_file.parent().ok_or_else(|| {
        syn::Error::new(
            selector.span(),
            "invoking source file has no parent directory",
        )
    })?;
    let path = std::path::absolute(parent.join(file)).map_err(|err| {
        syn::Error::new(selector.span(), format!("cannot resolve `{file}`: {err}"))
    })?;
    let text = fs::read_to_string(&path).map_err(|err| {
        syn::Error::new(
            selector.span(),
            format!("cannot read `{}`: {err}", path.display()),
        )
    })?;
    let body = markdown::extract(&text, section)
        .map_err(|message| syn::Error::new(selector.span(), format!("{file}: {message}")))?;
    Ok((body, path))
}

fn split_selector(value: &str) -> Result<(&str, &str), &'static str> {
    let (file, section) = value
        .split_once('#')
        .ok_or("expected `path/to/file.md#section`")?;
    if file.is_empty() || section.trim().is_empty() {
        return Err("file path and h2 section must both be nonempty");
    }
    Ok((file, section))
}

pub(crate) fn include(selector: &LitStr) -> syn::Result<TokenStream> {
    let (body, _) = load(selector)?;
    let literal = LitStr::new(&body, selector.span());
    Ok(quote!(#literal))
}

#[derive(Default)]
struct Expander {
    dependencies: BTreeSet<PathBuf>,
    errors: Option<syn::Error>,
}

impl Expander {
    fn doc(&mut self, selector: &LitStr) -> syn::Result<Attribute> {
        let (body, path) = load(selector)?;
        self.dependencies.insert(path);
        let body = LitStr::new(&body, selector.span());
        Ok(syn::parse_quote_spanned!(selector.span()=> #[doc = #body]))
    }

    fn record_error(&mut self, error: syn::Error) {
        if let Some(errors) = &mut self.errors {
            errors.combine(error);
        } else {
            self.errors = Some(error);
        }
    }
}

impl VisitMut for Expander {
    fn visit_attribute_mut(&mut self, attr: &mut Attribute) {
        let path = attr.path();
        let is_marker = path.is_ident("doc_section")
            || (path.segments.len() == 2
                && path.segments[0].ident == "doc_section"
                && path.segments[1].ident == "doc_section");
        if !is_marker {
            return;
        }
        let result = match &attr.meta {
            Meta::Path(_) => Ok(syn::parse_quote!(#[doc = ""])),
            Meta::List(list) if list.tokens.is_empty() => Ok(syn::parse_quote!(#[doc = ""])),
            Meta::List(_) => attr
                .parse_args::<LitStr>()
                .and_then(|selector| self.doc(&selector)),
            Meta::NameValue(_) => Err(syn::Error::new(
                attr.span(),
                "use #[doc_section(\"file.md#section\")]",
            )),
        };
        match result {
            Ok(mut replacement) => {
                replacement.style = attr.style;
                *attr = replacement;
            }
            Err(error) => self.record_error(error),
        }
    }
}

pub(crate) fn attribute(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let mut item: Item = syn::parse2(input)?;
    let attrs = match &mut item {
        Item::Struct(i) => &mut i.attrs,
        Item::Enum(i) => &mut i.attrs,
        Item::Union(i) => &mut i.attrs,
        Item::Fn(i) => &mut i.attrs,
        Item::Impl(i) => &mut i.attrs,
        Item::Trait(i) => &mut i.attrs,
        Item::Mod(i) if i.content.is_some() => &mut i.attrs,
        _ => {
            return Err(syn::Error::new(
                item.span(),
                "doc_section supports structs, enums, unions, functions, impls, traits and inline modules; for other items use #[doc = include_doc_section!(\"file.md#section\")] or annotate the enclosing inline module/impl/trait",
            ));
        }
    };
    let mut expander = Expander::default();
    if !args.is_empty() {
        let selector: LitStr = syn::parse2(args)?;
        attrs.insert(0, expander.doc(&selector)?);
    }
    expander.visit_item_mut(&mut item);
    if let Some(errors) = expander.errors {
        return Err(errors);
    }
    let mut tracking = TokenStream::new();
    for path in &expander.dependencies {
        let path = path
            .to_str()
            .ok_or_else(|| syn::Error::new(item.span(), "Markdown path is not UTF-8"))?;
        let path = LitStr::new(path, Span::mixed_site());
        tracking.extend(quote! { const _: &str = ::core::include_str!(#path); });
    }
    if let Item::Fn(function) = &mut item {
        // Local tracking also works when this function is a trait-impl method.
        use syn::parse::Parser;
        function
            .block
            .stmts
            .splice(0..0, syn::Block::parse_within.parse2(tracking)?);
        Ok(quote!(#item))
    } else {
        Ok(quote!(#item #tracking))
    }
}

#[cfg(test)]
mod tests {
    use super::split_selector;

    #[test]
    fn selectors_keep_namespace_and_spaces() {
        assert_eq!(
            split_selector("fruit.md#banana::peeling").unwrap(),
            ("fruit.md", "banana::peeling")
        );
        assert_eq!(
            split_selector("a b.md#Banana Peel").unwrap(),
            ("a b.md", "Banana Peel")
        );
    }

    #[test]
    fn malformed_selectors_fail() {
        for selector in ["fruit.md", "#banana", "fruit.md#", "fruit.md#  "] {
            assert!(split_selector(selector).is_err(), "{selector}");
        }
    }
}
