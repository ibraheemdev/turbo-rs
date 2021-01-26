use std::pin::Pin;
use bytes::Bytes;
use futures::Stream;

pub enum Body {
  Empty,
  Once(Bytes),
  Streamed(
    Pin<Box<dyn Stream<Item = Result<Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send>>,
  ),
}
