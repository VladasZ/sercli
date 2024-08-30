use crate::HeaderMap;

pub trait HeaderMapExt {
    fn store_raw_password(&self) -> bool;
}

impl HeaderMapExt for HeaderMap {
    fn store_raw_password(&self) -> bool {
        let Some(value) = self.get("store_raw_password") else {
            return false;
        };

        value
            .to_str()
            .expect("Failed to convert header value to string")
            .parse()
            .expect("Failed to convert header value to bool")
    }
}
