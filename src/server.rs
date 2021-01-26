use crate::http::{Request, Response};
use crate::turbofish::Turbofish;
use hyper::service::Service;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

impl Turbofish {
  async fn serve(self: Arc<Self>, req: Request) -> hyper::Result<Response> {
      self.router.serve(req).await
  }
}

pub(crate) struct MakeTurbofishService(TurbofishService);

impl MakeTurbofishService {
  pub fn new(t: Turbofish) -> Self {
    Self(TurbofishService(Arc::new(t)))
  }
}

impl<T> Service<T> for MakeTurbofishService {
  type Response = TurbofishService;
  type Error = hyper::Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

  fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, _: T) -> Self::Future {
    let service = self.0.clone();
    let fut = async move { Ok(service) };
    Box::pin(fut)
  }
}

#[derive(Clone)]
pub(crate) struct TurbofishService(Arc<Turbofish>);

impl Service<Request> for TurbofishService {
  type Response = Response;
  type Error = hyper::Error;
  type Future = Pin<Box<dyn Future<Output = hyper::Result<Response>> + Send>>;

  fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, req: Request) -> Self::Future {
      Box::pin(self.0.clone().serve(req))
  }
}
