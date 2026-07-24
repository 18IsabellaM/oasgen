use crate::{Schema, impl_oa_schema};

impl_oa_schema!(
    ::phonenumber::PhoneNumber,
    Schema::new_string().with_format("e164-phone-number")
);
