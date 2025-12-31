use ::std::{rc::Rc, sync::Arc};

use ::deno_core::{
    FastString, OpState, op2,
    v8::{self, Boolean, Local, PinScope, Value},
};
use ::rkyv::Archive;

use crate::{
    builtin::extension::syscall::SyscallError, config_value::ResolvedConfigValue,
    global_state::GlobalState, id::Label, package::ResolvedPackage,
};

#[derive(Debug, Clone)]
pub struct PackageInformation {
    pub package: ResolvedPackage,
    pub env: Arc<GlobalState>,
}

type InformationRc = Rc<PackageInformation>;

deno_core::extension!(
    zako_package,
    deps = [zako_context],
    esm_entry_point = "zako:package",
    esm = ["zako:package" = "../dist/builtins/package.js"],
    options = {
        info: InformationRc,
    },
    state = |state, options| {
        state.put(options.info);
    },
    docs = "The extension that provide package related APIs for zako",
);

#[op2]
#[to_v8]
fn syscall_package_group(state: &mut OpState) -> Result<FastString, SyscallError> {
    let info = state.borrow::<InformationRc>();
    let interner = info.env.interner();
    let group = info.package.group;
    interner
        .resolve(group)
        .map_err(SyscallError::from)
        .map(str::to_string)
        .map(FastString::from)
}

#[op2]
#[to_v8]
fn syscall_package_artifact(state: &mut OpState) -> Result<FastString, SyscallError> {
    let info = state.borrow::<InformationRc>();
    let interner = info.env.interner();
    let artifact = info.package.artifact;
    interner
        .resolve(artifact)
        .map_err(SyscallError::from)
        .map(str::to_string)
        .map(FastString::from)
}

#[op2]
#[to_v8]
fn syscall_package_version(state: &mut OpState) -> Result<FastString, SyscallError> {
    let info = state.borrow::<InformationRc>();
    let interner = info.env.interner();
    let version = info.package.version;
    interner
        .resolve(version)
        .map_err(SyscallError::from)
        .map(str::to_string)
        .map(FastString::from)
}

#[op2()]
fn syscall_package_config<'s, 'i>(
    scope: &mut v8::PinScope<'s, 'i>,
    state: &mut OpState,
    #[string] key: String,
) -> v8::Local<'s, v8::Value> {
    let info = state.borrow::<InformationRc>();
    let interner = info.env.interner();
    let value = info.package.config.config.iter().find_map(|label| {
        if let Ok(name) = label.0.resolved(interner)
            && name == key
        {
            Some(label.1.clone())
        } else {
            None
        }
    });

    match value {
        Some(value) => match value {
            ResolvedConfigValue::String(string) => v8::String::new(scope, string.as_str())
                .expect("failed to create v8 string")
                .into(),
            ResolvedConfigValue::Boolean(boolean) => v8::Boolean::new(scope, boolean).into(),
            ResolvedConfigValue::Number(number) => v8::Number::new(scope, number as f64).into(),
        },
        None => v8::undefined(scope).into(),
    }
}
