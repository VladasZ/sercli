use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Mutex, OnceLock},
};

static STATIC_API: OnceLock<API> = OnceLock::new();

#[derive(Debug)]
pub struct API {
    base_url: String,
    headers:  Mutex<HashMap<String, String>>,
}

impl API {
    pub fn init(base_url: impl Display) {
        STATIC_API
            .set(Self {
                base_url: format!("{base_url}"),
                headers:  Mutex::default(),
            })
            .unwrap();
    }

    pub fn is_ok() -> bool {
        STATIC_API.get().is_some()
    }

    fn get() -> &'static Self {
        STATIC_API.get().expect("API is not initialised. Use API::init(\"base_url\")")
    }
}

impl API {
    pub fn base_url() -> &'static str {
        &Self::get().base_url
    }

    pub fn headers() -> HashMap<String, String> {
        Self::get().headers.lock().unwrap().clone()
    }

    pub fn remove_header(key: impl ToString) {
        Self::get().headers.lock().unwrap().remove(&key.to_string());
    }

    pub fn add_header(key: impl ToString, value: impl ToString) {
        Self::get().headers.lock().unwrap().insert(key.to_string(), value.to_string());
    }
}
