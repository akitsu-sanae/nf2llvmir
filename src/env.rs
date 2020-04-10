use crate::ident::Ident;

#[derive(Debug, Clone)]
pub struct Env<T: Clone>(Vec<(Ident, T)>);

impl<T: Clone> Env<T> {
    pub fn new() -> Self {
        Env(vec![])
    }

    pub fn add(&self, name: Ident, v: T) -> Self {
        let mut new_env = self.clone();
        new_env.0.push((name, v));
        new_env
    }

    pub fn lookup(&self, name: &Ident) -> Option<T> {
        self.0
            .iter()
            .find(|e| &e.0 == name)
            .map(|res| res.clone().1)
    }
}
