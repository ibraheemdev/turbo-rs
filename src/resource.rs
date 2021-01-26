use crate::router::Route;

pub trait Resource {
    fn routes(&self) -> Vec<Route>;
}
