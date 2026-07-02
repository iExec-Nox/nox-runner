use opentelemetry::propagation::{Extractor, Injector};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub struct MessageHeaderExtractor<'a>(pub &'a async_nats::HeaderMap);

impl<'a> Extractor for MessageHeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.iter().map(|(k, _)| k.as_ref()).collect()
    }
}

pub struct HeaderInjector<'a>(pub &'a mut HeaderMap);

impl<'a> Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: std::string::String) {
        if let Ok(header_name) = HeaderName::from_bytes(key.as_bytes())
            && let Ok(header_value) = HeaderValue::from_str(&value)
        {
            self.0.insert(header_name, header_value);
        }
    }
}
