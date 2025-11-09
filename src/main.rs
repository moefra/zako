#![feature(inherent_associated_types)]

mod api;

use std::fs::File;
use clap::{CommandFactory, Parser, ValueEnum};
use clap::builder::styling;
use std::io;
use std::io::Write;
use clap_complete::{generate, shells};

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::BrightWhite.on_default())
    .error(styling::AnsiColor::BrightRed.on_default())
    .context(styling::AnsiColor::Blue.on_default())
    .context_value(styling::AnsiColor::BrightCyan.on_default())
    .valid(styling::AnsiColor::BrightGreen.on_default())
    .invalid(styling::AnsiColor::BrightYellow.on_default())
    .placeholder(styling::AnsiColor::Cyan.on_default().italic().bold());

#[derive(Parser, Debug)]
#[command(
    name = "zmake",
    bin_name = "zmake",
    version = env!("CARGO_PKG_VERSION"),
    about = "A building system",
    long_about = "The next-generation building tool that your mom warned you about.",
    styles = STYLES)]
enum Args {
    Information(InformationArgs),
    GenerateComplete(GenerateCompleteArgs),
}

#[derive(Debug,Clone,ValueEnum)]
enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

#[derive(clap::Args,Debug)]
#[command(name = "generate-complete",about="Generate shell completion file")]
struct GenerateCompleteArgs {
    #[arg(long)]
    shell:Shell,

    #[arg(long,default_value = "zmake")]
    bin_name:String,

    #[arg(long,default_value = None,help = "set this options to output to file,or it will output to stdout")]
    output_file:Option<String>
}
impl GenerateCompleteArgs{
    pub fn invoke(self){
        let mut command = Args::command();

        let bin_name = self.bin_name;

        let mut output : Box<dyn Write> =
            if let Some(file) = self.output_file{
                Box::new(File::open(file).unwrap())
            }
            else{
                Box::new(io::stdout())
            };

        match self.shell {
            Shell::Bash => {generate(shells::Bash, &mut command, bin_name, &mut output);}
            Shell::Elvish => {generate(shells::Elvish, &mut command, bin_name, &mut output);}
            Shell::Fish => {generate(shells::Fish, &mut command, bin_name, &mut output);}
            Shell::PowerShell => {generate(shells::PowerShell, &mut command, bin_name, &mut output);}
            Shell::Zsh => {generate(shells::Zsh, &mut command, bin_name, &mut output);}
        }
    }
}

use shadow_rs::{shadow, Format};
shadow!(build_information);
#[derive(clap::Args,Debug)]
#[command(name = "information",about="Print (debug) information about zmake")]
struct InformationArgs {}
impl InformationArgs{
    pub fn invoke(self){
        let local_time = shadow_rs::DateTime::now().human_format();
        println!("build local time:{local_time}");
        println!("is_debug:{}", shadow_rs::is_debug());
        println!("branch:{}", shadow_rs::branch());
        println!("tag:{}", shadow_rs::tag());
        println!("git_clean:{}", shadow_rs::git_clean());
        println!("git_status_file:{}", shadow_rs::git_status_file());
        println!();

        println!("version:{}", build_information::VERSION);
        println!("version:{}", build_information::CLAP_LONG_VERSION);
        println!("pkg_version:{}", build_information::PKG_VERSION);
        println!("pkg_version_major:{}", build_information::PKG_VERSION_MAJOR);
        println!("pkg_version_minor:{}", build_information::PKG_VERSION_MINOR);
        println!("pkg_version_patch:{}", build_information::PKG_VERSION_PATCH);
        println!("pkg_version_pre:{}", build_information::PKG_VERSION_PRE);
        println!();

        println!("tag:{}", build_information::TAG);
        println!("branch:{}", build_information::BRANCH);
        println!("commit_id:{}", build_information::COMMIT_HASH);
        println!("short_commit:{}", build_information::SHORT_COMMIT);
        println!("commit_date:{}", build_information::COMMIT_DATE);
        println!("commit_date_2822:{}", build_information::COMMIT_DATE_2822);
        println!("commit_date_3339:{}", build_information::COMMIT_DATE_3339);
        println!("commit_author:{}", build_information::COMMIT_AUTHOR);
        println!("commit_email:{}", build_information::COMMIT_EMAIL);
        println!();

        println!("build_os:{}", build_information::BUILD_OS);
        println!("rust_version:{}", build_information::RUST_VERSION);
        println!("rust_channel:{}", build_information::RUST_CHANNEL);
        println!("cargo_version:{}", build_information::CARGO_VERSION);
        println!("cargo_tree:{}", build_information::CARGO_TREE);
        println!();

        println!("project_name:{}", build_information::PROJECT_NAME);
        println!("build_time:{}", build_information::BUILD_TIME);
        println!("build_time_2822:{}", build_information::BUILD_TIME_2822);
        println!("build_time_3339:{}", build_information::BUILD_TIME_3339);
        println!("build_rust_channel:{}", build_information::BUILD_RUST_CHANNEL);
        println!();

        println!("=============== builtin constants ===============");
        println!("KAWAYI_GROUP_ID:{}", *api::builtin::zmake::KAWAYI_GROUP_ID);
        println!("ZMAKE_ARTIFACT_ID:{}", *api::builtin::zmake::ZMAKE_ARTIFACT_ID);
        println!("ZMAKE_QUALIFIED_ARTIFACT_ID:{}", *api::builtin::zmake::ZMAKE_QUALIFIED_ARTIFACT_ID);
        println!("ZMAKE_V1V0V0:{}", *api::builtin::zmake::ZMAKE_V1V0V0);
        println!();

        println!("LINUX:{}", (*api::builtin::os::LINUX).0);
        println!("MACOS:{}", (*api::builtin::os::MACOS).0);
        println!("WINDOWS:{}", (*api::builtin::os::WINDOWS).0);
        println!();

        println!("ARM64:{}", (*api::builtin::architecture::ARM64).0);
        println!("X64:{}", (*api::builtin::architecture::X64).0);
        println!();

        println!("ARCHIVER:{}", (*api::builtin::tool_type::ARCHIVER).0);
        println!();

        println!("INITIALIZE:{}", (*api::builtin::target_type::INITIALIZE).0);
        println!("BUILD:{}", (*api::builtin::target_type::BUILD).0);
        println!("CLEAN:{}", (*api::builtin::target_type::CLEAN).0);
        println!("TEST:{}", (*api::builtin::target_type::TEST).0);
        println!("PACKAGE:{}", (*api::builtin::target_type::PACKAGE).0);
        println!("INSTALL:{}", (*api::builtin::target_type::INSTALL).0);
        println!("DEPLOY:{}", (*api::builtin::target_type::DEPLOY).0);
        println!();
    }
}

fn main() {
    let args = Args::parse();

    match args {
        Args::Information(cmd) => cmd.invoke(),
        Args::GenerateComplete(cmd) => cmd.invoke(),
    }
}
