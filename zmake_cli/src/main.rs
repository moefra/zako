mod api;

use crate::api::id::ToolType;
use clap::builder::styling;
use clap::builder::styling::AnsiColor;
use clap::builder::styling::Color::Ansi;
use clap::{Arg, ColorChoice, CommandFactory, Parser, Subcommand, ValueEnum, arg, command};
use clap_complete::{generate, shells};
use const_format::concatcp;
use std::fs::File;
use std::io::Write;
use std::sync::LazyLock;
use std::thread::Builder;
use std::{env, io};
use tokio::runtime::Builder;

const STYLES: styling::Styles = styling::Styles::styled()
    .header(
        styling::AnsiColor::Green
            .on_default()
            .bg_color(Some(Ansi(AnsiColor::BrightWhite)))
            .bold(),
    )
    .usage(
        styling::AnsiColor::Green
            .on_default()
            .bg_color(Some(Ansi(AnsiColor::BrightWhite)))
            .bold(),
    )
    .literal(styling::AnsiColor::BrightWhite.on_default())
    .error(styling::AnsiColor::BrightRed.on_default())
    .context(styling::AnsiColor::Blue.on_default())
    .context_value(styling::AnsiColor::BrightCyan.on_default())
    .valid(styling::AnsiColor::BrightGreen.on_default())
    .invalid(styling::AnsiColor::BrightYellow.on_default())
    .placeholder(styling::AnsiColor::Cyan.on_default().italic().bold());

const ABOUT: &'static str =
    "The \x1b[35mpost-modern building tool\x1b[0m that your mom warned you about.";
const BEFORE_HELP: &'static str = concatcp!(
    "打碎旧世界,创立新世界.\n\x1B]8;;",
    env!("CARGO_PKG_HOMEPAGE"),
    "\x1B\\\x1b[34;47;4;1m[More Information]\x1B]8;;\x1B\\\x1b[0m"
);
const AFTER_HELP: &'static str = concatcp!(
    "早已森严壁垒,更加众志成城.\n\x1B]8;;",
    env!("CARGO_PKG_HOMEPAGE"),
    "\x1B\\\x1b[34;47;4;1m[Bug Report]\x1B]8;;\x1B\\\x1b[0m"
);

#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    bin_name = env!("CARGO_BIN_NAME"),
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = ABOUT,
    long_about = ABOUT,
    propagate_version = true,
    before_help = BEFORE_HELP,
    before_long_help = BEFORE_HELP,
    after_help = AFTER_HELP,
    after_long_help = AFTER_HELP,
    subcommand_help_heading = "Operations",
    styles = STYLES)]
struct Args {
    #[command(subcommand)]
    command: SubCommands,

    #[arg(
        global = true,
        group = "logging_level",
        long,
        help = "logging the most detailed information",
        visible_alias = "verbose"
    )]
    log_trace: bool,

    #[arg(
        global = true,
        group = "logging_level",
        long,
        help = "logging more detailed information"
    )]
    log_debug: bool,

    #[arg(
        global = true,
        group = "logging_level",
        long,
        help = "logging common information"
    )]
    log_information: bool,

    #[arg(
        global = true,
        group = "logging_level",
        long,
        help = "logging warnings only"
    )]
    log_warning: bool,

    #[arg(
        global = true,
        group = "logging_level",
        long,
        help = "logging errors only"
    )]
    log_error: bool,

    #[arg(
        global = true,
        group = "logging_level",
        long,
        help = "no logging",
        visible_alias = "quiet"
    )]
    log_off: bool,

    #[arg(
        value_enum,
        global = true,
        group = "logging_level",
        long,
        help = "set logging level",
        required = false,
        default_value = "info"
    )]
    log_level: Level,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    Information(InformationArgs),
    GenerateComplete(GenerateCompleteArgs),
    Make(MakeArgs),
}

#[derive(clap::Args, Debug)]
#[command(name = "make", about = "Build the project")]
struct MakeArgs {
    #[arg(long,default_value = "zmakefile.ts", value_hint = clap::ValueHint::FilePath)]
    projectFile: String,

    #[arg(long, help = "Set the cpu counts that zmake use")]
    concurrency: Option<usize>,
}

impl MakeArgs {
    pub fn invoke(self) {
        let concurrency = self.concurrency.unwrap_or(num_cpus::get());

        let builder = Builder::new_multi_thread();

        info!("use concurrency {}", concurrency)
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

#[derive(clap::Args, Debug)]
#[command(name = "generate-complete", about = "Generate shell completion file")]
struct GenerateCompleteArgs {
    #[arg(long)]
    shell: Shell,

    #[arg(long, default_value = "zmake")]
    bin_name: String,

    #[arg(long,default_value = None,help = "set this options to output to file,or it will output to stdout")]
    output_file: Option<String>,
}
impl GenerateCompleteArgs {
    pub fn invoke(self) {
        let mut command = Args::command();

        let bin_name = self.bin_name;

        let mut output: Box<dyn Write> = if let Some(file) = self.output_file {
            Box::new(File::open(file).unwrap())
        } else {
            Box::new(io::stdout())
        };

        match self.shell {
            Shell::Bash => {
                generate(shells::Bash, &mut command, bin_name, &mut output);
            }
            Shell::Elvish => {
                generate(shells::Elvish, &mut command, bin_name, &mut output);
            }
            Shell::Fish => {
                generate(shells::Fish, &mut command, bin_name, &mut output);
            }
            Shell::PowerShell => {
                generate(shells::PowerShell, &mut command, bin_name, &mut output);
            }
            Shell::Zsh => {
                generate(shells::Zsh, &mut command, bin_name, &mut output);
            }
        }
    }
}

use shadow_rs::{Format, shadow};
use tracing::{Level, info, trace_span};
use tracing_subscriber::FmtSubscriber;

shadow!(build_information);
#[derive(clap::Args, Debug)]
#[command(name = "information", about = "Print (debug) information about zmake")]
struct InformationArgs {}
impl InformationArgs {
    pub fn invoke(self) {
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
        println!(
            "build_rust_channel:{}",
            build_information::BUILD_RUST_CHANNEL
        );
        println!();

        println!("=============== builtin constants ===============");
        println!("KAWAYI_GROUP_ID:{}", *api::builtin::KAWAYI_GROUP_ID);
        println!("ZMAKE_ARTIFACT_ID:{}", *api::builtin::ZMAKE_ARTIFACT_ID);
        println!(
            "ZMAKE_QUALIFIED_ARTIFACT_ID:{}",
            *api::builtin::ZMAKE_QUALIFIED_ARTIFACT_ID
        );
        println!("ZMAKE_V1V0V0:{}", *api::builtin::ZMAKE_V1V0V0);
        println!();

        println!("LINUX:{}", (*api::builtin::LINUX).0);
        println!("MACOS:{}", (*api::builtin::MACOS).0);
        println!("WINDOWS:{}", (*api::builtin::WINDOWS).0);
        println!();

        println!("ARM64:{}", (*api::builtin::ARM64).0);
        println!("X64:{}", (*api::builtin::X64).0);
        println!();

        println!("ARCHIVER:{}", (*api::builtin::ARCHIVER).0);
        println!();

        println!("INITIALIZE:{}", (*api::builtin::INITIALIZE).0);
        println!("BUILD:{}", (*api::builtin::BUILD).0);
        println!("CLEAN:{}", (*api::builtin::CLEAN).0);
        println!("TEST:{}", (*api::builtin::TEST).0);
        println!("PACKAGE:{}", (*api::builtin::PACKAGE).0);
        println!("INSTALL:{}", (*api::builtin::INSTALL).0);
        println!("DEPLOY:{}", (*api::builtin::DEPLOY).0);
    }
}

fn main() {
    let _span = trace_span!("zmake start", version = env!("CARGO_PKG_VERSION")).entered();

    let _parseArgs = trace_span!("prase arguments").entered();
    let args = env::args_os();

    let args = argfile::expand_args_from(args, argfile::parse_fromfile, argfile::PREFIX).unwrap();

    let args = Args::parse_from(args);

    _parseArgs.exit();

    if !args.log_off {
        let subscriber = FmtSubscriber::builder()
            .with_ansi(true)
            .with_max_level(if args.log_trace {
                Level::TRACE
            } else if args.log_debug {
                Level::DEBUG
            } else if args.log_information {
                Level::INFO
            } else if args.log_warning {
                Level::WARN
            } else if args.log_error {
                Level::ERROR
            } else {
                args.log_level
            })
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    }

    match args.command {
        SubCommands::Information(cmd) => cmd.invoke(),
        SubCommands::GenerateComplete(cmd) => cmd.invoke(),
        SubCommands::Make(cmd) => cmd.invoke(),
    }
}
