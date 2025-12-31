use ::std::rc::Rc;

use ::deno_core::{FastString, OpState, ascii_str, op2};
use ::strum::IntoStaticStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
pub enum ContextName {
    Package,
}

/// The information of the context of the zako execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextInformation {
    pub name: ContextName,
}

pub type InformationRc = Rc<ContextInformation>;

deno_core::extension!(
    zako_context,
    deps = [zako_core],
    ops = [syscall_context_name],
    esm_entry_point = "zako:context",
    options = {
        info: InformationRc,
    },
    state = |state, options| {
        state.put(options.info);
    },
    docs = "The extension that provide context for zako execution",
);

#[op2]
#[to_v8]
fn syscall_context_name(state: &mut OpState) -> FastString {
    let info = state.borrow::<InformationRc>();
    match info.name {
        ContextName::Package => ascii_str!("package").into(),
    }
}
