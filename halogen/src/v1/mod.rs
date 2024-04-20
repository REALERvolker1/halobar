use serde::{Deserialize, Serialize};

#[cfg(feature = "ahash")]
use ahash::HashMap;
#[cfg(not(feature = "ahash"))]
use std::collections::HashMap;

/// A recognized type that can be sent through IPC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Variant {
    /// A literal string of text. Only meant to be used for Strings.
    ///
    /// Please use [`Variant::Other`] for custom data types.
    String(String),

    /// A boolean value, Accepts true/false, on/off, 1/0, case-insensitive
    Bool(bool),
    /// signed integer
    Iint(i64),
    /// Unsigned integer
    Uint(u64),
    /// Double-precision floating point number
    Float(f64),

    /// Multiple other Variants in a single-dimensional vector
    Vector(Vec<Box<Variant>>),
    /// Multiple other variants in a key-value map. The keys can only be Strings.
    Map(HashMap<String, Box<Variant>>),

    /// Another data type, possibly serialized as a String.
    Other(String),
}

#[cfg(test)]
mod tests {

    use super::*;
    #[cfg(feature = "ahash")]
    use ahash::HashMapExt;

    #[test]
    fn deserialize_variants() {
        let mut variants = HashMap::new();
        variants.insert(
            "string".to_owned(),
            Variant::String("Hello world!".to_owned()),
        );
        variants.insert("bool".to_owned(), Variant::Bool(true));
        variants.insert("signed int".to_owned(), Variant::Iint(-673485));
        variants.insert("unsigned_int".to_owned(), Variant::Uint(678656397));
        variants.insert("float".to_owned(), Variant::Float(-3778.489));

        variants.insert(
            "vector".to_owned(),
            Variant::Vector(vec![
                Box::new(Variant::Iint(-785)),
                Box::new(Variant::String("Hello World".to_owned())),
            ]),
        );

        let mut map = HashMap::new();
        map.insert(
            "name".to_owned(),
            Box::new(Variant::String("Drew".to_owned())),
        );
        map.insert("age".to_owned(), Box::new(Variant::Uint(32)));

        variants.insert("map".to_owned(), Variant::Map(map));

        let json = serde_json::to_string_pretty(&variants).unwrap();

        let from_json: HashMap<String, Variant> = serde_json::from_str(&json).unwrap();

        assert_eq!(from_json, variants);
    }
}
