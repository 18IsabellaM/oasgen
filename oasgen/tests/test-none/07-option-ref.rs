use oasgen::OaSchema;
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
}

#[derive(OaSchema, Serialize, Deserialize)]
pub struct Name {
    first: String,
    last: String,
}

#[derive(OaSchema, Serialize, Deserialize)]
pub struct Person {
    // `Option<Enum>` renders as a `$ref`, so it must be wrapped in `allOf` with
    // `nullable: true` on the wrapper.
    sex: Option<Sex>,
    // `Option<NestedStruct>` also renders as a `$ref` and must be wrapped.
    name: Option<Name>,
    // `Option<String>` renders inline, so `nullable: true` is set directly.
    nickname: Option<String>,
    // Non-`Option` field is the only one that appears in `required`.
    age: i32,
}

fn main() {
    use pretty_assertions::assert_eq;
    let schema = Person::schema();
    let spec = serde_yaml::to_string(&schema).unwrap();
    assert_eq!(spec.trim(), include_str!("07-option-ref.yaml"));
}
