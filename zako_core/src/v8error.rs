use crate::module_specifier::ModuleSpecifier;

/// The means a V8 error that can not continue.
///
/// For some exception that can continue,do not use this,and use Value as an exception to broadcast the error in js.
///
/// It was usually used as:
/// ```rust
/// Result<Result<(),Local<Value>>,V8Error>.
/// //-----success^-------^js exception ^fatal error that can not continue
/// ```
#[derive(Clone,Debug,Default)]
pub struct V8Error{
    pub module_specifier:Option<ModuleSpecifier>,
    pub message:String,
    pub stack:Option<String>,
    pub exception: Option<String>,
    pub addition_message:Option<String>,
}
