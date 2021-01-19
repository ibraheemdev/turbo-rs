use crate::http::{Request, Response};

pub trait Action {
    fn call(&self, req: Request) -> Response;
}

pub type BoxedAction = Box<dyn Action + Send + Sync>;
