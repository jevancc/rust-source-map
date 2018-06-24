extern crate linked_hash_map;
extern crate regex;
extern crate vlq;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

mod mapping;
mod mapping_list;
mod source_map;
mod source_map_generator;
mod source_node;
mod utils;

pub use mapping::Mapping;
pub use source_map_generator::*;
pub use source_node::*;
use std::rc::Rc;

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
