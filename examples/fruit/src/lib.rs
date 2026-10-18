#![doc = doc_section::include_doc_section!("fruit.md#overview")]

use doc_section::doc_section;

#[doc_section("fruit.md#banana")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Banana {
    #[doc_section("fruit.md#banana_thickness")]
    pub thickness: f32,
}

#[doc_section("fruit.md#orange")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orange {
    #[doc_section("fruit.md#orange_sugar_content")]
    pub sugar_content: f32,
}

#[doc_section("fruit.md#peeled_banana")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeeledBanana;

impl Banana {
    #[doc_section("fruit.md#banana::new")]
    pub const fn new(thickness: f32) -> Self {
        Self { thickness }
    }

    #[doc_section("fruit.md#banana::peeling")]
    pub fn peeling(&self) -> PeeledBanana {
        PeeledBanana
    }
}
