use crate::http::{Request, Response};

#[crate::async_trait]
pub trait Action {
    async fn call(&self, req: Request) -> Response;
}

pub type BoxedAction = Box<dyn Action + Send + Sync>;
