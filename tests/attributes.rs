use doc_section::{doc_section, include_doc_section};

#[doc_section("sections.md#item")]
#[derive(Debug, PartialEq)]
struct Generic<T: Copy> {
    #[doc_section("sections.md#field")]
    value: T,
}

#[doc_section]
struct Tuple(#[doc_section("sections.md#field")] u8);

#[doc_section]
#[allow(dead_code)]
union Number {
    #[doc_section("sections.md#field")]
    integer: u32,
    float: f32,
}

#[doc_section("sections.md#item")]
mod nested {
    #[doc_section("sections.md#item")]
    #[derive(Debug, PartialEq)]
    pub enum Fruit {
        #[doc_section("sections.md#value")]
        Banana {
            #[doc_section("sections.md#field")]
            thickness: u8,
        },
        #[doc_section::doc_section("sections.md#value")]
        Orange(#[doc_section("sections.md#field")] u8),
    }

    #[doc_section]
    pub trait Value {
        #[doc_section("sections.md#method")]
        fn value(&self) -> u8;
    }

    #[doc_section]
    impl Value for Fruit {
        #[doc_section("sections.md#method")]
        fn value(&self) -> u8 {
            match self {
                Self::Banana { thickness } => *thickness,
                Self::Orange(sugar) => *sugar,
            }
        }
    }

    #[doc_section]
    impl Fruit {
        #[doc_section("sections.md#value")]
        pub const DEFAULT: u8 = 3;
    }
}

trait IntoNumber {
    fn into_number(self) -> u8;
}

impl IntoNumber for Tuple {
    #[doc_section("sections.md#method")]
    fn into_number(self) -> u8 {
        self.0
    }
}

#[doc_section]
trait WithAssociated {
    #[doc_section("sections.md#value")]
    type Output;
    #[doc_section("sections.md#value")]
    const VALUE: u8;
    #[doc_section("sections.md#method")]
    fn value(&self) -> Self::Output;
}

#[doc_section]
impl WithAssociated for Tuple {
    #[doc_section("sections.md#value")]
    type Output = u8;
    #[doc_section("sections.md#value")]
    const VALUE: u8 = 7;
    #[doc_section("sections.md#method")]
    fn value(&self) -> Self::Output {
        self.0
    }
}

#[doc_section("sections.md#method")]
const fn const_function() -> u8 {
    42
}

#[doc_section("sections.md#method")]
async fn async_function() -> u8 {
    43
}

#[test]
fn preserves_types_methods_generics_and_derives() {
    use nested::{Fruit, Value};
    assert_eq!(Generic { value: 5 }, Generic { value: 5 });
    assert_eq!(Tuple(8).into_number(), 8);
    assert_eq!(Tuple(9).value(), 9);
    assert_eq!(Tuple::VALUE, 7);
    assert_eq!(Fruit::Banana { thickness: 3 }.value(), 3);
    assert_eq!(Fruit::Orange(12).value(), 12);
    assert_eq!(Fruit::DEFAULT, 3);
    let object: &dyn Value = &Fruit::Orange(4);
    assert_eq!(object.value(), 4);
    const ANSWER: u8 = const_function();
    assert_eq!(ANSWER, 42);
    drop(async_function());
}

#[test]
fn includes_string_literal() {
    const TEXT: &str = include_doc_section!("sections.md#field");
    assert_eq!(TEXT, "A field documented from Markdown.");
}
