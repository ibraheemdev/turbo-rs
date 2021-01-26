use crate::http::{Method, Uri, HeaderValue, HeaderMap, CookieJar};

/// The HTTP request header consists of a method, uri, cookie jar, and a set of
/// header fields.
pub struct RequestHeader {
    method: Method,
    uri: Uri,
    headers: HeaderMap<HeaderValue>,
    cookies: CookieJar
}

impl RequestHeader {
    pub fn method(&self) -> &Method {
        &self.method
    }
}
