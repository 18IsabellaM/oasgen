use crate::OaSchema;
use http::{Method, Uri, Version};
use openapiv3::Schema;

impl OaSchema for Method {
    fn schema() -> Schema {
        Schema::new_string()
    }
}

impl OaSchema for Version {
    fn schema() -> Schema {
        Schema::new_string()
    }
}

impl OaSchema for Uri {
    fn schema() -> Schema {
        Schema::new_string()
    }
}
