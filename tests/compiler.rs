//! Real consumer compilation verifies diagnostics, rustdoc output and rebuilds.
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

struct Consumer {
    path: PathBuf,
}

impl Consumer {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("doc-section-{}-{nonce}", std::process::id()));
        fs::create_dir_all(path.join("src/nested")).unwrap();
        fs::create_dir_all(path.join("docs")).unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        fs::write(path.join("Cargo.toml"), format!(
            "[package]\nname = \"doc-section-consumer\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n[dependencies]\ndoc-section = {{ path = {root:?} }}\n"
        )).unwrap();
        Self { path }
    }

    fn write(&self, name: &str, contents: &str) {
        fs::write(self.path.join(name), contents).unwrap();
    }

    fn cargo(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO"))
            .current_dir(&self.path)
            .env("CARGO_TARGET_DIR", self.path.join("target"))
            .args(args)
            .arg("--offline")
            .output()
            .unwrap()
    }

    fn doc(&self) {
        let out = self.cargo(&["doc", "--no-deps"]);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn html(&self, name: &str) -> String {
        fs::read_to_string(self.path.join("target/doc/doc_section_consumer").join(name)).unwrap()
    }

    fn fails(&self, source: &str, diagnostic: &str) {
        self.write("src/lib.rs", source);
        let out = self.cargo(&["check"]);
        assert!(!out.status.success(), "expected failure for {source}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(diagnostic),
            "expected {diagnostic:?}\n{stderr}"
        );
        assert!(
            stderr.contains("src/lib.rs:"),
            "diagnostic must point at the selector: {stderr}"
        );
        assert!(!stderr.contains("proc macro panicked"), "{stderr}");
    }
}

impl Drop for Consumer {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn consumer_docs_diagnostics_and_markdown_rebuilds() {
    let project = Consumer::new();
    project.write("docs/fruit.md", "# Fruit\n\n## banana\nBanana original.\n\n### Examples\n```rust\nassert_eq!(1, 1);\n```\n\n## field\nField original.\n\n## ignored\nUNRELATED SECTION\n");
    project.write("docs/method.md", "## peeling\nMethod original.\n");
    project.write(
        "src/lib.rs",
        r##"#![no_std]
pub mod nested;
"##,
    );
    project.write(
        "src/nested/mod.rs",
        r##"
#[doc_section::doc_section("../../docs/fruit.md#banana")]
pub struct Banana {
    #[doc_section("../../docs/fruit.md#field")]
    pub thickness: f32,
}

impl Banana {
    #[doc_section::doc_section("../../docs/method.md#peeling")]
    pub fn peeling(&self) -> f32 { self.thickness }
}
"##,
    );
    project.doc();
    let html = project.html("nested/struct.Banana.html");
    assert!(html.contains("Banana original."));
    assert!(html.contains("Field original."));
    assert!(html.contains("Method original."));
    assert!(
        html.contains("<h2 id=\"examples\""),
        "rustdoc promotes Markdown h1 to HTML h2"
    );
    assert!(!html.contains("UNRELATED SECTION"));

    // No Rust edits and no build.rs: attribute expansion must register the file.
    project.write(
        "docs/fruit.md",
        "## banana\nBanana revised and longer.\n\n## field\nField revised and longer.\n",
    );
    project.write("docs/method.md", "## peeling\nMethod revised and longer.\n");
    project.doc();
    let html = project.html("nested/struct.Banana.html");
    assert!(html.contains("Banana revised and longer."));
    assert!(html.contains("Field revised and longer."));
    assert!(html.contains("Method revised and longer."));
    assert!(!html.contains("Banana original."));

    // Isolate method-only dependency tracking from the struct's other file.
    project.write(
        "docs/method.md",
        "## peeling\nMethod changed independently.\n",
    );
    project.doc();
    assert!(
        project
            .html("nested/struct.Banana.html")
            .contains("Method changed independently.")
    );

    // Crate-level literal macro has an explicit build-script dependency.
    project.write(
        "build.rs",
        "fn main() { println!(\"cargo::rerun-if-changed=docs/fruit.md\"); }",
    );
    project.write(
        "src/lib.rs",
        r##"#![no_std]
#![doc = doc_section::include_doc_section!("../docs/fruit.md#banana")]
"##,
    );
    project.doc();
    assert!(
        project
            .html("index.html")
            .contains("Banana revised and longer.")
    );
    project.write("docs/fruit.md", "## banana\nCrate docs revised again.\n");
    project.doc();
    assert!(
        project
            .html("index.html")
            .contains("Crate docs revised again.")
    );

    project.fails(
        "#[doc_section::doc_section(\"../docs/fruit.md#absent\")] pub struct Missing;",
        "h2 section `absent` not found",
    );
    project.fails(
        "#[doc_section::doc_section(\"../docs/fruit.md\")] pub struct Bad;",
        "expected `path/to/file.md#section`",
    );
    project.fails(
        "#[doc_section::doc_section(12)] pub struct Bad;",
        "expected string literal",
    );
    project.fails(
        "#[doc_section::doc_section(\"../docs/absent.md#banana\")] pub struct Missing;",
        "cannot read",
    );
    project.write("docs/fruit.md", "## banana\none\n## banana\ntwo\n");
    project.fails(
        "#[doc_section::doc_section(\"../docs/fruit.md#banana\")] pub struct Duplicate;",
        "duplicate headings",
    );
    project.fails("#[doc_section::doc_section] mod nested;", "inline modules");
    project.fails("#[doc_section::doc_section] struct Bad { #[doc_section(\"../docs/fruit.md#absent\")] x: u8 }", "h2 section `absent` not found");
    project.fails(
        "#[doc_section::doc_section] struct Bad { #[doc_section = \"bad\"] x: u8 }",
        "use #[doc_section(",
    );
    project.fails(
        "#![doc = doc_section::include_doc_section!(\"../docs/fruit.md#absent\")]",
        "h2 section `absent` not found",
    );
}
