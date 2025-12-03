use crate::id::Id;
use std::rc::Rc;

pub struct Target {
    id: Id,
    public_dependencies: Vec<Id>,
    private_dependencies: Vec<Rc<Target>>,
    tasks: Vec<Box<u32>>,
}
