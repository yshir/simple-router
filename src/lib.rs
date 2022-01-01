use derive_new::new;

#[derive(Default, Debug)]
pub struct Router {
    nodes: Vec<Node>,
}

impl Router {
    pub fn route(&self, method: Method, pattern: &str, handler: Handler) -> Self {
        // trailing slash
        let mut pattern = pattern.to_string();
        if pattern.ends_with('/') {
            pattern.pop();
        }

        let mut nodes = self.nodes.clone();
        nodes.push(Node::new(method, pattern, handler));
        Self { nodes }
    }

    pub fn get(&self, pattern: &str, handler: Handler) -> Self {
        self.route(Method::GET, pattern, handler)
    }
    pub fn post(&self, pattern: &str, handler: Handler) -> Self {
        self.route(Method::POST, pattern, handler)
    }
    pub fn put(&self, pattern: &str, handler: Handler) -> Self {
        self.route(Method::PUT, pattern, handler)
    }
    pub fn delete(&self, pattern: &str, handler: Handler) -> Self {
        self.route(Method::DELETE, pattern, handler)
    }

    pub fn resolve(&self, method: &str, path: &str) -> String {
        for node in &self.nodes {
            let method = Method::try_from(method).unwrap();
            if node.method == method {
                let path = {
                    let mut a = path.to_string();

                    // remove consecutive slashes
                    // /foo////bar -> /foo/bar
                    while a.contains("//") {
                        a = a.replace("//", "/");
                    }

                    // trailing slash
                    // /foo/ -> /foo
                    if a.ends_with('/') {
                        a.pop();
                    }

                    a
                };

                // /foo/bar -> /foo/bar
                // /foo/*/bar -> /foo/a/bar, /foo/b/bar, ...
                let paths = path.split('/');
                let node_paths = node.pattern.split('/');
                if paths.clone().count() == node_paths.clone().count() {
                    let ok = paths
                        .zip(node_paths)
                        .all(|(str, node_str)| str == node_str || node_str == "*");
                    if ok {
                        return (node.handler)();
                    }
                }
            }
        }

        String::from("no match routes")
    }
}

#[derive(new, Debug, Clone)]
pub struct Node {
    method: Method,
    pattern: String,
    handler: Handler,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

impl TryFrom<&str> for Method {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "GET" | "get" => Ok(Method::GET),
            "POST" | "post" => Ok(Method::POST),
            "PUT" | "put" => Ok(Method::PUT),
            "DELETE" | "delete" => Ok(Method::DELETE),
            _ => Err("invalid method"),
        }
    }
}

pub type Handler = fn() -> String;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_adds_node() {
        let router = Router::default();
        assert_eq!(0, router.nodes.len());

        let router = Router::default()
            .route(Method::GET, "/foo", || String::from("foo"))
            .route(Method::GET, "/bar", || String::from("bar"));
        assert_eq!(2, router.nodes.len())
    }

    #[test]
    fn resolve_returns_a_string() {
        let router = Router::default()
            .route(Method::GET, "/get", || String::from("get"))
            .route(Method::POST, "/post", || String::from("post"))
            .put("/put", || String::from("put"))
            .delete("/delete", || String::from("delete"));

        assert_eq!("get", router.resolve("GET", "/get"));
        assert_eq!("post", router.resolve("POST", "/post"));
        assert_eq!("put", router.resolve("PUT", "/put"));
        assert_eq!("delete", router.resolve("DELETE", "/delete"));
        assert_eq!("no match routes", router.resolve("GET", "/foo"));
    }

    #[test]
    fn resolve_placeholder() {
        let router = Router::default()
            .route(Method::GET, "/foo/*", || String::from("foo"))
            .route(Method::GET, "/foo/*/*/bar", || String::from("foobar"));

        assert_eq!("foo", router.resolve("GET", "/foo/1"));
        assert_eq!("foo", router.resolve("GET", "/foo/a"));
        assert_eq!("foobar", router.resolve("GET", "/foo/1/2/bar"));
        assert_eq!("no match routes", router.resolve("GET", "/foo/1/2/3/bar"));
        assert_eq!("no match routes", router.resolve("GET", "/foo/1/2/bar/3"));
    }

    #[test]
    fn consecutive_slashes_ignored() {
        let router = Router::default().route(Method::GET, "/a/b/c", || String::from("abc"));

        assert_eq!("abc", router.resolve("GET", "/a//////b//c"));
    }

    #[test]
    fn trailing_slash() {
        let router = Router::default()
            .get("/foo", || String::from("foo"))
            .get("/bar/", || String::from("bar"));

        assert_eq!("foo", router.resolve("GET", "/foo"));
        assert_eq!("foo", router.resolve("GET", "/foo/"));
        assert_eq!("foo", router.resolve("GET", "/foo//"));
        assert_eq!("bar", router.resolve("GET", "/bar"));
        assert_eq!("bar", router.resolve("GET", "/bar/"));
        assert_eq!("bar", router.resolve("GET", "/bar//"));
    }
}
