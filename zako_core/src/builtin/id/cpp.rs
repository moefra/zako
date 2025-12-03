use crate::make_builtin_id;

make_builtin_id! {
    self => {
        "moe.kawayi:zmake@1.0.0" => {
            ToolType =>
            {
                COMPILER => "cpp/compiler",
                PREPROCESSOR => "cpp/preprocessor"
            },
            Tool =>{
                GCC => "cpp/gcc",
                CLANG => "cpp/clang",
                MSVC => "cpp/msvc"
            }
        }
    }
}
