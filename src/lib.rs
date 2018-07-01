extern crate linked_hash_map;
extern crate regex;
extern crate vlq;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod mapping;
mod mapping_list;
mod source_map;
mod source_map_consumer;
mod source_map_generator;
mod source_node;
mod utils;

pub use mapping::Mapping;
pub use source_map_generator::*;
pub use source_node::*;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Node {
    NSourceNode(SourceNode),
    NString(String),
    NRcString(Rc<String>),
    NNodeVec(Vec<Node>),
}

#[derive(Clone)]
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

    pub fn get(&self) -> &str {
        match self {
            StringPtr::Str(s) => &s,
            StringPtr::Ptr(p) => p,
        }
    }
}
