use std::collections::HashMap;

use openapiv3::{
    IntegerFormat, IntegerType, ReferenceOr, Schema, SchemaData, SchemaKind, Type,
    VariantOrUnknownOrEmpty,
};

#[cfg(feature = "actix")]
mod actix;

#[cfg(feature = "axum")]
mod axum;

#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "cookies")]
mod cookies;
#[cfg(feature = "phonenumber")]
mod phonenumber;
#[cfg(feature = "sqlx")]
mod sqlx;
#[cfg(feature = "time")]
mod time;

#[cfg(feature = "bigdecimal")]
mod bigdecimal;
mod http;
#[cfg(feature = "sid")]
mod sid;
mod tuple;

pub trait OaSchema {
    fn schema() -> Schema;

    fn schema_ref() -> ReferenceOr<Schema> {
        ReferenceOr::Item(Self::schema())
    }
    /// You should rarely if ever implement this method.
    #[doc(hidden)]
    fn body_schema() -> Option<ReferenceOr<Schema>> {
        Some(Self::schema_ref())
    }
}

pub struct SchemaRegister {
    pub name: &'static str,
    pub constructor: &'static (dyn Sync + Send + Fn() -> Schema),
}

inventory::collect!(SchemaRegister);

#[macro_export]
macro_rules! impl_oa_schema {
    ($t:ty,$schema:expr) => {
        impl $crate::OaSchema for $t {
            fn schema() -> $crate::Schema {
                $schema
            }
        }
    };
}

#[macro_export]
macro_rules! impl_oa_schema_passthrough {
    ($t:ty) => {
        impl<T> $crate::OaSchema for $t
        where
            T: $crate::OaSchema,
        {
            fn schema_ref() -> $crate::ReferenceOr<$crate::Schema> {
                T::schema_ref()
            }

            fn schema() -> $crate::Schema {
                T::schema()
            }
        }
    };
}

impl_oa_schema!(bool, Schema::new_bool());

/// Builds an integer schema carrying the OpenAPI format and the inclusive value
/// range of the Rust type it represents.
fn integer_schema(format: IntegerFormat, minimum: i64, maximum: i64) -> Schema {
    Schema {
        data: SchemaData::default(),
        kind: SchemaKind::Type(Type::Integer(IntegerType {
            format: VariantOrUnknownOrEmpty::Item(format),
            minimum: Some(minimum),
            maximum: Some(maximum),
            ..IntegerType::default()
        })),
    }
}

// OpenAPI bounds are `i64`, so the upper bound of `u64` and `usize` cannot be
// expressed. Clamping to `i64::MAX` keeps the schema valid and still rejects the
// values most likely to be generated out of range.
const UNSIGNED_64_MAX: i64 = i64::MAX;

impl_oa_schema!(
    usize,
    integer_schema(IntegerFormat::Int64, 0, UNSIGNED_64_MAX)
);
impl_oa_schema!(
    isize,
    integer_schema(IntegerFormat::Int64, i64::MIN, i64::MAX)
);

impl_oa_schema!(
    u8,
    integer_schema(IntegerFormat::Int32, u8::MIN as i64, u8::MAX as i64)
);
impl_oa_schema!(
    i8,
    integer_schema(IntegerFormat::Int32, i8::MIN as i64, i8::MAX as i64)
);

impl_oa_schema!(
    u16,
    integer_schema(IntegerFormat::Int32, u16::MIN as i64, u16::MAX as i64)
);
impl_oa_schema!(
    i16,
    integer_schema(IntegerFormat::Int32, i16::MIN as i64, i16::MAX as i64)
);

// `u32::MAX` exceeds `i32::MAX`, so the wider `int64` format is the only one that
// can describe the full range.
impl_oa_schema!(
    u32,
    integer_schema(IntegerFormat::Int64, u32::MIN as i64, u32::MAX as i64)
);
impl_oa_schema!(
    i32,
    integer_schema(IntegerFormat::Int32, i32::MIN as i64, i32::MAX as i64)
);

impl_oa_schema!(
    u64,
    integer_schema(IntegerFormat::Int64, 0, UNSIGNED_64_MAX)
);
impl_oa_schema!(
    i64,
    integer_schema(IntegerFormat::Int64, i64::MIN, i64::MAX)
);

impl_oa_schema!(f32, Schema::new_number());
impl_oa_schema!(f64, Schema::new_number());

impl_oa_schema!(String, Schema::new_string());

impl<T> OaSchema for Vec<T>
where
    T: OaSchema,
{
    fn schema() -> Schema {
        let inner = T::schema();
        Schema::new_array(inner)
    }

    fn schema_ref() -> ReferenceOr<Schema> {
        let inner = T::schema_ref();
        ReferenceOr::Item(Schema::new_array(inner))
    }
}

impl<T> OaSchema for Option<T>
where
    T: OaSchema,
{
    fn schema() -> Schema {
        let mut schema = T::schema();
        schema.nullable = true;
        schema
    }

    fn schema_ref() -> ReferenceOr<Schema> {
        let mut schema = T::schema_ref();
        match schema.as_mut() {
            // Inline schema: set `nullable` directly on it.
            Some(s) => {
                s.nullable = true;
                schema
            }
            // The inner type renders as a `$ref`. OpenAPI 3.0.x ignores keywords
            // placed as siblings of `$ref`, so wrap the reference in `allOf` and
            // set `nullable` on the wrapper to preserve the `Option`'s nullability.
            None => {
                let mut wrapper = Schema::new_all_of(vec![schema]);
                wrapper.nullable = true;
                ReferenceOr::Item(wrapper)
            }
        }
    }
}

impl OaSchema for () {
    fn schema() -> Schema {
        panic!("Unit type has no schema")
    }
    fn body_schema() -> Option<ReferenceOr<Schema>> {
        None
    }
}

impl<K, V> OaSchema for HashMap<K, V>
where
    V: OaSchema,
{
    fn schema() -> Schema {
        Schema::new_map(V::schema())
    }

    fn schema_ref() -> ReferenceOr<Schema> {
        ReferenceOr::Item(Schema::new_map(V::schema_ref()))
    }
}

#[cfg(feature = "uuid")]
impl_oa_schema!(uuid::Uuid, Schema::new_string().with_format("uuid"));

impl_oa_schema!(serde_json::Value, Schema::new_object());

#[cfg(test)]
mod tests {
    use super::*;

    fn integer_type<T: OaSchema>() -> IntegerType {
        match T::schema().kind {
            SchemaKind::Type(Type::Integer(i)) => i,
            other => panic!("expected an integer schema, got {other:?}"),
        }
    }

    #[test]
    fn integer_primitives_carry_format_and_bounds() {
        let expected = [
            ("i8", integer_type::<i8>(), IntegerFormat::Int32, -128, 127),
            ("u8", integer_type::<u8>(), IntegerFormat::Int32, 0, 255),
            (
                "i16",
                integer_type::<i16>(),
                IntegerFormat::Int32,
                -32768,
                32767,
            ),
            ("u16", integer_type::<u16>(), IntegerFormat::Int32, 0, 65535),
            (
                "i32",
                integer_type::<i32>(),
                IntegerFormat::Int32,
                i32::MIN as i64,
                i32::MAX as i64,
            ),
            (
                "u32",
                integer_type::<u32>(),
                IntegerFormat::Int64,
                0,
                u32::MAX as i64,
            ),
            (
                "i64",
                integer_type::<i64>(),
                IntegerFormat::Int64,
                i64::MIN,
                i64::MAX,
            ),
            (
                "isize",
                integer_type::<isize>(),
                IntegerFormat::Int64,
                i64::MIN,
                i64::MAX,
            ),
        ];

        for (name, actual, format, minimum, maximum) in expected {
            assert_eq!(
                actual.format,
                VariantOrUnknownOrEmpty::Item(format),
                "{name} format"
            );
            assert_eq!(actual.minimum, Some(minimum), "{name} minimum");
            assert_eq!(actual.maximum, Some(maximum), "{name} maximum");
        }
    }

    #[test]
    fn unsigned_64_bit_maxima_are_clamped_to_the_representable_range() {
        for (name, actual) in [
            ("u64", integer_type::<u64>()),
            ("usize", integer_type::<usize>()),
        ] {
            assert_eq!(
                actual.format,
                VariantOrUnknownOrEmpty::Item(IntegerFormat::Int64),
                "{name} format"
            );
            assert_eq!(actual.minimum, Some(0), "{name} minimum");
            assert_eq!(actual.maximum, Some(i64::MAX), "{name} maximum");
        }
    }
}
