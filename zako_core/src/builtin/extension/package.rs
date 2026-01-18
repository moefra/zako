use ::std::{rc::Rc, sync::Arc};

use ::deno_core::{
    FastString, OpState, op2,
    v8::{self, Boolean, Local, PinScope, Value},
};
use ::rkyv::Archive;
use smol_str::SmolStr;

use crate::{
    builtin::extension::syscall::SyscallError,
    config::{ConfigError, Configuration},
    config_value::{ConfigDefault, ConfigType, ConfigValue, ResolvedConfigValue},
    global_state::GlobalState,
    id::Label,
    intern::Interner,
    package::{Package, ResolvedPackage, ResolvingPackage},
};

#[derive(Debug, Clone)]
pub struct PackageInformation {
    package: ResolvingPackage,
    config: Configuration,
}

impl PackageInformation {
    pub fn new(package: ResolvingPackage, interner: &Interner) -> Result<Self, ConfigError> {
        Ok(Self {
            config: package.resolved_config.resolve(interner)?,
            package,
        })
    }

    pub fn get_package(self) -> ResolvingPackage {
        self.package
    }
}

pub type InformationRc = Rc<PackageInformation>;

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
    let group = info.package.original.group.clone();
    Ok(group.to_string().into())
}

#[op2]
#[to_v8]
fn syscall_package_artifact(state: &mut OpState) -> Result<FastString, SyscallError> {
    let info = state.borrow::<InformationRc>();
    let artifact = info.package.original.artifact.clone();
    Ok(artifact.to_string().into())
}

#[op2]
#[to_v8]
fn syscall_package_version(state: &mut OpState) -> Result<FastString, SyscallError> {
    let info = state.borrow::<InformationRc>();
    let version = info.package.original.version.clone();
    Ok(version.to_string().into())
}

#[op2()]
fn syscall_package_config<'s, 'i>(
    scope: &mut v8::PinScope<'s, 'i>,
    state: &mut OpState,
    #[string] key: String,
) -> v8::Local<'s, v8::Value> {
    let info = state.borrow::<InformationRc>();

    let config = &info.config;

    let key = SmolStr::new(key);

    let value = config.config.get(&key);

    match value {
        Some(value) => match &value.default {
            ConfigDefault::String(string) => v8::String::new(scope, string.as_str())
                .expect("failed to create v8 string")
                .into(),
            ConfigDefault::Object(_) => unreachable!(),
            ConfigDefault::Boolean(boolean) => v8::Boolean::new(scope, *boolean).into(),
            ConfigDefault::Number(number) => v8::Number::new(scope, *number as f64).into(),
        },
        None => v8::undefined(scope).into(),
    }
}
