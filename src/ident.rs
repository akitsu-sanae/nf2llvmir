use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(pub String);

impl Ident {
    pub fn new(name: &str) -> Ident {
        Ident(name.to_string())
    }

    pub fn fresh_tuple_name() -> Ident {
        Ident(format!("tuple{}", COUNTER.fetch_add(1, SeqCst)))
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);
