use std::rc::Rc;
use crate::api::id::Id;

pub struct Target{
    id:Id,
    public_dependencies:Vec<Id>,
    private_dependencies:Vec<Rc<Target>>,
    tasks:Vec<Box<u32>>,
}
