#[macro_export]
macro_rules! make_builtin_id {
    {
        $( pub mod $submodule:ident; )*
        self => { $($id:literal => { $($type:ident => { $($key:ident => $value:literal),* }),* } ),* }
    } => {
        #[allow(unused_imports)]
        use $crate::id::IdType::{ToolType,Tool,ToolProvider,TargetType,Target,Os,Architecture,Property,Config};

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
                    pub static $key : $crate::id::ResolvedId = {
                        let id = $id;
                        let id_type: &'static str = $type.into();
                        let id_str = format!("{}#{}::{}", id,id_type,$value);
                        <$crate::id::ResolvedId as ::std::str::FromStr>::from_str(&id_str).expect("Builtin ID format error")
                    };
                )*
            )*
        )*

        #[::static_init::dynamic(lazy)]
        pub static BUILTIN: ::std::collections::BTreeMap<::std::string::String, crate::id::ResolvedId> = {
            let mut map = ::std::collections::BTreeMap::<::std::string::String, $crate::id::ResolvedId>::new();

            $(
                let id = $id;

                $(
                    let id_type: &'static str = crate::id::IdType::$type.into();

                    $(
                        let value = format!("{}#{}::{}", id,id_type,$value);
                        let value = <$crate::id::ResolvedId as ::std::str::FromStr>::from_str(&value).unwrap();

                        let key = ::convert_case::ccase!(camel, std::stringify!($key));

                        map.insert(key.to_string(),value);
                    )*
                )*
            )*

            map
        };
    }
}
