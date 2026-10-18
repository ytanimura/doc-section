# doc-section

[![crates.io](https://img.shields.io/crates/v/doc-section)](https://crates.io/crates/doc-section)
[![docs.rs](https://img.shields.io/docsrs/doc-section)](https://docs.rs/doc-section)

Keep your Rust API documentation in one Markdown file. `doc-section` extracts individual level-two heading sections and turns them into `#[doc = "..."]` attributes, so you can document a module, its types, fields, and methods without creating a separate file for every item.

Targets stable Rust 1.88 or newer.

## Getting started

Add `doc-section` to your `Cargo.toml`:

```toml
[dependencies]
doc-section = "0.1"
```

For local development against a checkout, use a path dependency instead:

```toml
[dependencies]
doc-section = { path = "../doc-section" }
```

Place `fruit.md` next to the Rust source file that references it:

````markdown
# Fruit

## overview

A module defining fruit types.

## banana

A banana characterized by its peel thickness.

## banana_thickness

The thickness of the banana's peel.

## banana::peeling

Peels the banana.

### Examples

```rust
use fruit::*;

let banana = Banana::new(3.0);
banana.peeling();
```
````

Use `#[doc_section("file.md#section")]` to attach a section to an item:

```rust,ignore
use doc_section::doc_section;

#[doc_section("fruit.md#banana")]
pub struct Banana {
    #[doc_section("fruit.md#banana_thickness")]
    pub thickness: f32,
}

impl Banana {
    #[doc_section("fruit.md#banana::peeling")]
    pub fn peeling(&self) -> PeeledBanana {
        // Implement the method here.
    }
}
```

The attribute on the struct also processes its field attributes. To document only the fields, apply `#[doc_section]` to the struct without an argument. Existing documentation, derives, visibility, and generics are preserved.

The selected h2 heading is omitted, and headings within its body move up two levels. For example, `### Examples` becomes `# Examples`, as expected by rustdoc.

See [examples/fruit](https://github.com/ytanimura/doc-section/tree/main/examples/fruit) for a complete working crate, including the constructor, return type, and executable documentation example.

## Modules, implementations, and traits

Apply the attribute to an inline module to process the documentation markers inside it, including those on structs, fields, enum variants, methods, associated types, and associated constants. You can also apply `#[doc_section]` without an argument to an `impl`, `trait`, or `enum`.

```rust,ignore
#[doc_section::doc_section("fruit.md#overview")]
pub mod fruit {
    #[doc_section("fruit.md#banana")]
    pub struct Banana {
        #[doc_section("fruit.md#banana_thickness")]
        pub thickness: f32,
    }
}
```

Nested markers must be spelled `doc_section` or `doc_section::doc_section`. Aliased markers, markers inside `cfg_attr`, and tokens passed to other macros are not rewritten.

The attribute macro can be applied directly to structs, enums, unions, functions or methods with bodies, `impl` blocks, traits, and inline modules. To document trait methods without bodies, associated types, or associated constants, apply it to the enclosing `trait` or `impl`.

Out-of-line module declarations such as `#[doc_section(...)] mod fruit;` are not supported. Apply the attribute to items within the module's source file instead, or use the literal macro below for the module's own documentation.

## Crate and module documentation

Rust does not allow custom inner attribute macros such as `#![doc_section(...)]`. For documentation at the start of a crate or module source file, use `include_doc_section!`, which expands to a string literal:

```rust,ignore
#![doc = doc_section::include_doc_section!("fruit.md#overview")]
```

You can also use this macro in a field's `#[doc = ...]` attribute or anywhere a string literal is accepted.

### Tracking Markdown changes

**When using `include_doc_section!`, register each Markdown file in the consuming crate's `build.rs`.** Stable Rust does not provide a direct dependency-tracking API for procedural macros that return only a string literal. Without this registration, changing only the Markdown file may leave stale documentation in an incremental build.

Use paths relative to the consuming crate's `Cargo.toml`:

```rust,ignore
fn main() {
    println!("cargo::rerun-if-changed=src/fruit.md");
}
```

Files processed by the `#[doc_section(...)]` or `#[doc_section]` attribute are tracked automatically through generated anonymous constants containing `include_str!`; they do not need a build script.

An `include_doc_section!` invocation still requires the build-script registration even when it appears inside an item annotated with `#[doc_section]`.

## Section extraction rules

- In `file.md#section`, the file path is **relative to the Rust source file containing the selector string**. Absolute paths are also accepted.
- The selector matches the rendered text of a top-level h2 heading exactly. Matching is case-sensitive and does not generate URL slugs. Names such as `banana::peeling`, names containing spaces, and Unicode names are supported.
- Both ATX headings (`## section`) and Setext h2 headings are supported. Extraction starts after the selected heading and ends before the next top-level h1 or h2 heading. Leading and trailing newline characters are removed.
- Top-level h3 through h6 headings within the body move up two levels: `### Examples` becomes `# Examples`, and `#### Details` becomes `## Details`. Other Markdown source is preserved.
- Parsing follows CommonMark. Apparent headings inside fenced code blocks, indented code blocks, or HTML blocks are ignored. Headings inside blockquotes or lists neither delimit sections nor change levels.
- Missing sections, duplicate selected headings, invalid selectors, and file-reading failures produce compile errors at the selector. Empty sections are allowed.
- Link destinations are not rewritten. Reference-link definitions and footnotes outside the selected section are not imported. Keep each section self-contained.

For Rust's attribute macro restrictions, see the [Rust Reference](https://doc.rust-lang.org/reference/procedural-macros.html). Source-relative path resolution uses [Span::local_file](https://doc.rust-lang.org/proc_macro/struct.Span.html#method.local_file), stabilized in Rust 1.88.

## Development

The workspace contains the procedural macro crate and the [Fruit example crate](https://github.com/ytanimura/doc-section/tree/main/examples/fruit). The example covers crate documentation, structs, fields, methods, and an executable `Examples` section.

```sh
cargo test --workspace
cargo doc --workspace --no-deps --open
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Tests cover section extraction, attribute expansion, compiler diagnostics, generated rustdoc content, and incremental rebuilds after Markdown-only changes. Japanese test strings intentionally exercise Unicode support.

## License

Licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0). See the `LICENSE` file for the full license text.
