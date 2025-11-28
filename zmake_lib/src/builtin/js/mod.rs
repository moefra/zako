use v8::FunctionTemplate;

use crate::{make_builtin_js, module_loader::ModuleLoadError, module_specifier::ModuleSpecifier};

pub static RT_CODE: &'static str = concat!(std::env!("CARGO_MANIFEST_DIR"), "/../dist/rt.js");

#[::static_init::dynamic(lazy)]
pub static RT: ModuleSpecifier = ModuleSpecifier::Builtin("rt".to_string());

#[::static_init::dynamic(lazy)]
pub static SYSCALL: ModuleSpecifier = ModuleSpecifier::Builtin("syscall".to_string());

/*
 *  To modify the name of method,remeber to modify it in js file too.
 */
make_builtin_js!(
    syscalls:{
        get_zmake_version
    }
    accessors:
    {
        zmake_version
    }
);

pub fn get_zmake_version<'s, 'i>(
    scope: &mut ::v8::PinScope<'s, 'i>,
    _args: ::v8::FunctionCallbackArguments<'s>,
    mut return_value: ::v8::ReturnValue<'s, v8::Value>,
) {
    let version = v8::String::new(scope, env!("CARGO_PKG_VERSION")).unwrap();
    return_value.set(version.into());
}

pub fn zmake_version<'s, 'i>(
    scope: &mut ::v8::PinScope<'s, 'i>,
) -> Result<v8::Local<'s, v8::Value>, ModuleLoadError> {
    Ok(v8::String::new(scope, env!("CARGO_PKG_VERSION"))
        .ok_or_else(|| {
            crate::module_loader::ModuleLoadError::V8ObjectAllocationError(
                "failed to create string",
            )
        })?
        .into())
}
