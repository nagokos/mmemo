use crate::app::{
    config::SelectorKind,
    selector::{
        builtin::Builtin,
        external::{Fzf, Skim},
    },
};

pub mod builtin;
pub mod core;
pub mod external;

pub trait Selector {
    fn select(&self, items: Vec<String>) -> std::io::Result<Option<String>>;
}

pub fn selector_select(selector: &SelectorKind) -> Box<dyn Selector> {
    match selector {
        SelectorKind::Builtin => Box::new(Builtin),
        SelectorKind::Fzf => Box::new(Fzf),
        SelectorKind::Skim => Box::new(Skim),
    }
}
