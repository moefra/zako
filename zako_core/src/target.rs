use crate::id::{Id, ResolvedId};
use std::rc::Rc;

pub struct Target {
    id: ResolvedId,
    public_dependencies: Vec<ResolvedId>,
    private_dependencies: Vec<Rc<Target>>,
    tasks: Vec<Box<u32>>,
}
