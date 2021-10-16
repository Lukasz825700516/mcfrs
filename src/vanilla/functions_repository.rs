use std::ops::RangeFrom;

use super::{function::Function, namespace::Namespace};

pub struct FunctionsRepository<'a> {
    pub namespace: &'a Namespace<'a>,

    pub functions: Vec<Function<'a>>,
    pub next_function_id: RangeFrom<u32>,
}


impl<'a> FunctionsRepository<'a> {
    pub fn new(namespace: &'a Namespace) -> Self {
        Self {
            namespace,

            functions: Vec::new(),
            next_function_id: 0..,
        }
    }
}
