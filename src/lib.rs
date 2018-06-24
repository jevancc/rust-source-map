extern crate vlq;
extern crate linked_hash_map;
extern crate regex;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

mod source_node;
mod source_map_generator;
mod mapping;
mod mapping_list;
mod source_map;
mod utils;

use std::rc::Rc;
pub use source_node::*;
pub use source_map_generator::*;

pub enum Node {
    NSourceNode(SourceNode),
    NString(String),
    NRcString(Rc<String>),
    NNodeVec(Vec<Node>),
}

pub enum StringPtr {
    Str(String),
    Ptr(Rc<String>),
}

impl StringPtr {
    pub fn to_ptr(self) -> Rc<String> {
        match self {
            StringPtr::Str(s) => Rc::new(s),
            StringPtr::Ptr(p) => p,
        }
    }
}
