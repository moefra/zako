use crate::module_loader::ModuleLoadError;

#[macro_export]
macro_rules! make_builtin_id {
    {
        $( pub mod $submodule:ident; )*
        self => { $($id:literal => { $($type:ident => { $($key:ident => $value:literal),* }),* } ),* }
    } => {
        #[allow(unused_imports)]
        use $crate::id::IdType::{ToolType,Tool,ToolProvider,TargetType,Target,Os,Architecture,Property};

        $(
            pub mod $submodule;
        )*

        #[::static_init::dynamic(lazy)]
        pub static TYPESCRIPT_EXPORT: ::std::string::String = {
            let mut typescript = ::std::string::String::new();

            for (key,value) in BUILTIN.iter(){
                typescript.push_str(&format!("\t{}: \"{}\",\n",key,value));
            }

            $(
                typescript.push_str(&format!("\t{}: {{\n", ::std::stringify!($submodule)));

                for (key,value) in $submodule::BUILTIN.iter(){
                    typescript.push_str(&format!("\t\t{}: \"{}\",\n",key,value));
                }

                typescript.push_str("\t},\n");
            )*

            typescript
        };

        $(
            $(
                $(
                    #[::static_init::dynamic(lazy)]
                    pub static $key : $crate::id::Id = {
                        let id = $id;
                        let id_type: &'static str = $type.into();
                        let id_str = format!("{}#{}::{}", id,id_type,$value);
                        <$crate::id::Id as ::std::str::FromStr>::from_str(&id_str).expect("Builtin ID format error")
                    };
                )*
            )*
        )*

        #[::static_init::dynamic(lazy)]
        pub static BUILTIN: ::std::collections::BTreeMap<::std::string::String, crate::id::Id> = {
            let mut map = ::std::collections::BTreeMap::<::std::string::String, $crate::id::Id>::new();

            $(
                let id = $id;

                $(
                    let id_type: &'static str = crate::id::IdType::$type.into();

                    $(
                        let value = format!("{}#{}::{}", id,id_type,$value);
                        let value = <$crate::id::Id as ::std::str::FromStr>::from_str(&value).unwrap();

                        let key = ::convert_case::ccase!(camel, std::stringify!($key));

                        map.insert(key.to_string(),value);
                    )*
                )*
            )*

            map
        };
    }
}

/// 我想叫他syscall，但是ai认为不妥。
///
/// 还是用我的吧。
pub type Syscall = for<'s, 'i> fn(
    &mut ::v8::PinScope<'s, 'i>,
    ::v8::FunctionCallbackArguments<'s>,
    ::v8::ReturnValue<'s, ::v8::Value>,
);

pub type SysAccessor = for<'s, 'i> fn(
    &mut ::v8::PinScope<'s, 'i>,
) -> Result<v8::Local<'s, v8::Value>, ModuleLoadError>;

#[macro_export]
macro_rules! make_builtin_js {
    ( syscalls: { $($syscall:ident),* } accessors: { $($accessor:ident),* } ) => {
        #[::static_init::dynamic(lazy)]
        pub static BUILTIN_SYSCALLS: ::std::collections::BTreeMap<::std::string::String, $crate::make_builtin::Syscall> = {
            let mut map = ::std::collections::BTreeMap::<::std::string::String, $crate::make_builtin::Syscall>::new();

            $(
                map.insert(::std::format!("{}", std::stringify!(syscall)), $syscall as $crate::make_builtin::Syscall).unwrap();
            )*

            map
        };

        #[::static_init::dynamic(lazy)]
        pub static BUILTIN_ACCESSORS: ::std::collections::BTreeMap<::std::string::String, $crate::make_builtin::SysAccessor> = {
            let mut map = ::std::collections::BTreeMap::<::std::string::String, $crate::make_builtin::SysAccessor>::new();

            $(
                map.insert(::std::format!("{}", std::stringify!(accessor)), $accessor as $crate::make_builtin::SysAccessor).unwrap();
            )*

            map
        };

        pub fn set_syscalls<'s,'i>(scope: &mut ::v8::PinScope<'s, 'i>,module: &::v8::Local<'s,::v8::Module>)->
            std::result::Result<(),$crate::module_loader::ModuleLoadError>{
                $(
                let function = ::v8::FunctionTemplate::new(scope, $syscall)
                    .get_function(scope)
                    .ok_or_else(|| {
                    $crate::module_loader::ModuleLoadError::V8ObjectAllocationError("failed to create function")
                })?;

                module.set_synthetic_module_export(
                    scope,
                    ::v8::String::new(scope, ::std::stringify!($syscall))
                    .ok_or_else(|| {
                        $crate::module_loader::ModuleLoadError::V8ObjectAllocationError("failed to create function name")
                    })?,
                    function.into()
                    )
                    .ok_or_else(|| {
                        $crate::module_loader::ModuleLoadError::V8SyntheticModuleBuildingError(::std::stringify!($syscall))
                    })?;
                )*
            Ok(())
        }

        pub fn set_accessors<'s,'i>(scope: &mut ::v8::PinScope<'s, 'i>,module: &::v8::Local<'s,::v8::Module>)->
            std::result::Result<(),$crate::module_loader::ModuleLoadError>{

                $(
                    let accessor = $accessor(scope)?;

                    module.set_synthetic_module_export(
                        scope,
                        ::v8::String::new(scope, ::std::stringify!($accessor))
                        .ok_or_else(|| {
                            $crate::module_loader::ModuleLoadError::V8ObjectAllocationError("failed to create accessor name")
                    })?,
                        accessor
                    )
                    .ok_or_else(|| {
                        $crate::module_loader::ModuleLoadError::V8SyntheticModuleBuildingError(::std::stringify!($syscall))
                    })?;
                )*

            Ok(())
        }

        pub fn get_exports<'s,'i>(scope: &mut ::v8::PinScope<'s, 'i>) ->
                ::std::result::Result<::std::vec::Vec<::v8::Local<'s, ::v8::String>>,$crate::module_loader::ModuleLoadError>{
            let mut exports = ::std::vec::Vec::<::v8::Local<'s, ::v8::String>>::new();

            for syscall in BUILTIN_SYSCALLS.keys(){
                exports.push(::v8::String::new(scope, syscall)
                            .ok_or_else(|| {
                            $crate::module_loader::ModuleLoadError::V8ObjectAllocationError("failed to create syscall name")
                            })?);
            }
            for accessor in BUILTIN_ACCESSORS.keys(){
                exports.push(::v8::String::new(scope, accessor)
                            .ok_or_else(|| {
                            $crate::module_loader::ModuleLoadError::V8ObjectAllocationError("failed to create accessor name")
                            })?);
            }

            Ok(exports)
        }

        pub fn evalution_callback<'a>(context:v8::Local<'a, ::v8::Context>, module:v8::Local<'a, ::v8::Module>)
            -> ::std::option::Option<::v8::Local<'a, ::v8::Value>>{
                // unsafe这一块
                ::v8::callback_scope!(unsafe scope, context);

                match set_syscalls(scope,&module){
                    Err(err) => {
                        ::tracing::error!("failed to evaluate module:{}",err);
                        return None;
                    },
                    Ok(()) => {}
                }
                match set_accessors(scope,&module){
                    Err(err) => {
                        ::tracing::error!("failed to evaluate module:{}",err);
                        return None;
                    },
                    Ok(()) => {}
                }

                Some(::v8::undefined(scope).into())
        }
    };
}
