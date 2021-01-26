pub(crate) mod tree;

use crate::action::BoxedAction;
use crate::http::{Method, Request, Response, Body, StatusCode};
use crate::resource::Resource;
use std::collections::HashMap;
use tree::Match;

pub struct Route {
	name: &'static str,
	controller: &'static str,
	method: Method,
	handler: BoxedAction,
	path: &'static str,
}

impl Route {
	pub async fn call(&self, req: Request) -> Response {
		self.handler.call(req).await
	}
}

pub struct Router {
	routes: HashMap<Method, tree::Node<Route>>,
}

impl Default for Router {
	fn default() -> Self {
		Self {
			routes: HashMap::with_capacity(5),
		}
	}
}

impl Router {
	pub async fn serve(&self, req: Request) -> hyper::Result<Response> {
		let root = self.routes.get(req.method());
		let path = req.uri().path();
		if let Some(root) = root {
			match root.match_path(path) {
				Ok(lookup) => {
					req.extensions_mut().insert(lookup.params);
					return Ok(lookup.value.call(req).await);
				}
				Err(tsr) => {
					if req.method() != Method::CONNECT && path != "/" {
						let code = match *req.method() {
							// Moved Permanently, request with GET method
							Method::GET => StatusCode::MOVED_PERMANENTLY,
							// Permanent Redirect, request with same method
							_ => StatusCode::PERMANENT_REDIRECT,
						};

						if tsr {
							let path = if path.len() > 1 && path.ends_with('/') {
								path[..path.len() - 1].to_string()
							} else {
								path.to_string() + "/"
							};

							return Ok(
								Response::builder()
									.header(header::LOCATION, path.as_str())
									.status(code)
									.body(Body::empty())
									.unwrap(),
							);
						};

						if let Some(fixed_path) = root.find_case_insensitive_path(&clean(path), true) {
							return Ok(
								Response::builder()
									.header(header::LOCATION, fixed_path.as_str())
									.status(code)
									.body(Body::empty())
									.unwrap(),
							);
						}
					};
				}
			}
		};

		if req.method() == Method::OPTIONS {
			let allow = self.allowed(path).join(", ");
			if allow != "" {
				return Ok(
					Response::builder()
						.header(header::ALLOW, allow)
						.body(Body::empty())
						.unwrap(),
				);
			}
		} else {
			let allow = self.allowed(path).join(", ");

			if !allow.is_empty() {
				return Ok(
					Response::builder()
						.header(header::ALLOW, allow)
						.status(StatusCode::METHOD_NOT_ALLOWED)
						.body(Body::empty())
						.unwrap(),
				);
			}
		};

		Ok(Response::builder().status(404).body(Body::empty()).unwrap())
	}

	pub fn resource(&mut self, resource: impl Resource) {
		for route in resource.routes() {
			self.route(route);
		}
	}

	pub fn node(&self, method: &Method) -> Option<&tree::Node<Route>> {
		self.routes.get(method)
	}

	pub fn route(&mut self, route: Route) {
		self
			.routes
			.entry(route.method.clone())
			.or_insert_with(tree::Node::default)
			.insert(route.path, route);
	}

	pub fn lookup(&self, method: &Method, path: &str) -> Result<Match<Route>, bool> {
		self
			.routes
			.get(method)
			.map_or(Err(false), |r| r.match_path(path))
	}
}
