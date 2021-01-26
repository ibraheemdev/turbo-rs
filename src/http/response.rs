use crate::http::{HeaderMap, HeaderValue, StatusCode, Body};

pub struct Response {
  status: StatusCode,
  headers: HeaderMap<HeaderValue>,
  body: Body,
}
