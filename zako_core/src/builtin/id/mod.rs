use crate::make_builtin_id;

make_builtin_id! {
    pub mod c;
    pub mod cpp;

    self => {
        "moe.kawayi:zmake@1.0.0" => {
            TargetType =>
            {
                INITIALIZE => "initialize",
                CLEAN => "clean",
                BUILD => "build",
                TEST => "test",
                PACKAGE => "package",
                INSTALL => "install",
                DEPLOY => "deploy"
            },
            Architecture =>{
                X64 => "x64",
                ARM64 => "arm64"
            },
            Os =>{
                WINDOWS => "windows",
                LINUX => "linux",
                MACOS => "macos"
            },
            ToolType =>{
                ARCHIVER => "archiver",
                DOWNLOADER => "downloader",
                LINKER => "linker"
            },
            Tool =>{
                LINK_EXE => "link_exe",
                LD => "ld",
                BSD_TAR => "bad_tar",
                CURL => "curl",
                GIT => "git"
            }
        }
    }
}

pub fn construct_builtins_typescript_export() -> String {
    let mut result = String::from("export default {\n");
    //result.push_str(&TYPESCRIPT_EXPORT);
    result.push_str("}\n");
    result
}
