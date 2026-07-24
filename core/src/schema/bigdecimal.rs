use crate::{Schema, impl_oa_schema};

impl_oa_schema!(
    ::bigdecimal::BigDecimal,
    Schema::new_string().with_format("decimal")
);
