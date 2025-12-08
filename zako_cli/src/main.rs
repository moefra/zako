mod buffered_writer;

use crate::buffered_writer::TemporaryBufferedWriterMaker;
use clap::builder::styling;
use clap::builder::styling::AnsiColor;
use clap::builder::styling::Color::Ansi;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum, arg, command};
use clap_complete::{generate, shells};
use color_eyre::owo_colors::OwoColorize;
use colorchoice::ColorChoice;
use const_format::concatcp;
use eyre::eyre;
use opentelemetry::trace::TracerProvider;
use sha2::Digest;
use shadow_rs::{Format, shadow};
use std::env::temp_dir;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::{env, io};
use tokio::runtime::Builder;
use tracing::{Level, debug, error, info, trace_span};
use tracing::{Span, trace};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;
use tracing_tree::HierarchicalLayer;
use zako_core::engine::{Engine, EngineMode, EngineOptions};
use zako_core::file_finder::FileFinder;
use zako_core::path::NeutralPath;
use zako_core::project_resolver::ProjectResolver;

const STYLES: styling::Styles = styling::Styles::styled()
    .header(
        styling::AnsiColor::Green
            .on_default()
            .bg_color(Some(Ansi(AnsiColor::BrightWhite)))
            .bold()
            .italic(),
    )
    .usage(
        styling::AnsiColor::Green
            .on_default()
            .bg_color(Some(Ansi(AnsiColor::BrightRed)))
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
    "The \x1b[35mpost-modern building tool\x1b[0m🛠️ that your mom warned you about🤯";
const BEFORE_HELP: &'static str = concatcp!(
    "打碎💨旧世界⚰️创立🚀新世界❤️‍🔥\n\x1B]8;;",
    env!("CARGO_PKG_HOMEPAGE"),
    "\x1B\\\x1b[34;47;4;1m[More Information]\x1B]8;;\x1B\\\x1b[0m"
);
const AFTER_HELP: &'static str = concatcp!(
    "Support argfile(namely @response_file), use @ARG_FILE to load arguments from file📄\n\n",
    "早已森严壁垒🧱更加众志成城💪\n\x1B]8;;",
    env!("CARGO_PKG_HOMEPAGE"),
    "\x1B\\\x1b[34;47;4;1m[Bug Report]\x1B]8;;\x1B\\\x1b[0m"
);

#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    bin_name = env!("CARGO_BIN_NAME"),
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    flatten_help = true,
    propagate_version = true,
    about = ABOUT,
    long_about = ABOUT,
    before_help = BEFORE_HELP,
    before_long_help = BEFORE_HELP,
    after_help = AFTER_HELP,
    after_long_help = AFTER_HELP,
    styles = STYLES,
    subcommand_help_heading = "Operations")]
struct Args {
    #[command(subcommand)]
    command: SubCommands,

    #[arg(
        global = true,
        long,
        help = "this will print backtrace and spans but do not set log level"
    )]
    backtrace: bool,

    #[arg(
        global = true,
        long,
        visible_alias = "quiet",
        help = "suppress all output"
    )]
    silent: bool,

    #[command(flatten)]
    color: colorchoice_clap::Color,

    /// Change to DIRECTORY before doing anything
    #[arg(
        short = 'C',
        long = "directory",
        global = true,
        value_name = "DIR",
        help = "Change to DIRECTORY before doing anything,but argfile is still read in current directory,not changed directory"
    )]
    pub chdir: Option<std::path::PathBuf>,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    Information(InformationArgs),
    GenerateComplete(GenerateCompleteArgs),
    ExportBuiltin(ExportBuiltinArgs),
    Make(MakeArgs),
    Bun(BunArgs),
}

fn run_program(name: &str, binary: &[u8], args: Vec<String>) -> Result<(), eyre::Error> {
    let _span = trace_span!("execute program", name = name, args = format!("{:?}", args)).entered();

    let home = dirs::cache_dir().unwrap_or(temp_dir()).join(
        format!(
            "{}-{}-bin-{}",
            env!("CARGO_BIN_NAME"),
            name,
            env!("CARGO_PKG_VERSION")
        )
        .as_str(),
    );

    #[cfg(target_os = "windows")]
    let exe = home.join(format!("{}.exe", name));
    #[cfg(not(target_os = "windows"))]
    let exe = home.join(name);

    match fs::create_dir_all(&home) {
        Ok(_) => (),
        Err(e) => {
            return Err(eyre::eyre!(
                "failed to create program home directory `{:?}`: {}",
                home,
                e
            ));
        }
    };

    if !exe.exists() {
        debug!("Extracting embedded binary to {:?}", exe);

        let temp_exe = home.join(format!(".tmp-{}-{}", name, std::process::id()));

        {
            let mut file = fs::File::create(&temp_exe)?;
            file.write_all(binary)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = file.metadata()?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&temp_exe, perms)?;
            }
            file.sync_all()?;
        }

        match fs::rename(&temp_exe, &exe) {
            Ok(_) => {}
            Err(e) => {
                let _ = fs::remove_file(&temp_exe);
                if !exe.exists() {
                    return Err(eyre::eyre!("Failed to rename program binary: {}", e));
                }
            }
        }
    }

    let mut exe = std::process::Command::new(&exe);
    let exe = exe
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .current_dir(std::env::current_dir()?);

    let path = env::var("PATH").unwrap_or_else(|_| "".to_string());
    let path = env::split_paths(&path).map(|x| OsString::from(x));
    let path = std::env::join_paths([PathBuf::from(home), path.collect()].iter())?;
    exe.env("PATH", path);

    let status = exe.status()?;

    if !status.success() {
        return Err(eyre::eyre!(
            "failed to execute program(exit code: {})",
            status
                .code()
                .map(|x| x.to_string())
                .unwrap_or("unknown".to_string())
        ));
    }

    Ok(())
}

#[derive(clap::Args, Debug)]
#[command(
    name = "bun",
    about = "Execute bun command,relay following arguments to bun",
    disable_help_flag = true,
    disable_version_flag = true
)]
struct BunArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}
static BUN_BINARY_ZSTD: &[u8] = include_bytes!(concat!(std::env!("OUT_DIR"), "/bun.zst"));

fn decompress_zstd(data: &[u8]) -> eyre::Result<Vec<u8>> {
    let mut decoder = zstd::stream::Decoder::new(data)?;
    let mut decompressed_data = Vec::new();
    std::io::copy(&mut decoder, &mut decompressed_data)?;
    Ok(decompressed_data)
}

fn run_bun(args: Vec<String>) -> eyre::Result<()> {
    let decompressed_binary = decompress_zstd(BUN_BINARY_ZSTD)?;
    run_program("bun", &decompressed_binary, args)
}

#[derive(clap::Args, Debug)]
#[command(
    name = "export-builtin",
    about = "Export builtin typescript variable to file or stdout"
)]
struct ExportBuiltinArgs {
    #[arg(long, value_hint = clap::ValueHint::FilePath)]
    output_file: Option<String>,
}

impl ExportBuiltinArgs {
    pub fn invoke(self) -> eyre::Result<()> {
        let builtins = zako_core::builtin::id::construct_builtins_typescript_export();

        if let Some(output_file) = self.output_file {
            let mut output_file = File::create(output_file)?;

            output_file.write_all(builtins.as_bytes())?;
        } else {
            println!("{}", builtins);
        }
        Ok(())
    }
}

#[derive(clap::Args, Debug)]
#[command(name = "make", about = "Build the project")]
struct MakeArgs {
    /// The file path must be valid NeutralPath.
    ///
    /// It will join into the sandbox path to construct the full path.
    #[arg(long,default_value = ::zako_core::PROJECT_FILE_NAME, value_hint = clap::ValueHint::FilePath)]
    project_file: String,

    /// The path to construct the sandbox
    #[arg(long,default_value = ".", value_hint = clap::ValueHint::DirPath)]
    sandbox_dir: String,

    #[arg(long, help = "Set the cpu counts to use")]
    concurrency: Option<usize>,
}

impl MakeArgs {
    pub fn invoke(self) -> eyre::Result<()> {
        let concurrency = self.concurrency.unwrap_or(num_cpus::get());

        let runtime = Builder::new_multi_thread().build()?;

        let project_file = NeutralPath::new(self.project_file)?;

        info!("use concurrency {}", concurrency);

        let engine = Engine::new(EngineOptions {
            tokio_handle: runtime.handle().clone(),
            mode: EngineMode::Project,
        })?;

        let mut resolver = ProjectResolver::new(engine);

        resolver.resolve_project(&project_file)?;

        Ok(())
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

    #[arg(long, default_value = env!("CARGO_BIN_NAME"))]
    bin_name: String,

    #[arg(long,default_value = None,help = "set this options to output to file,or it will output to stdout")]
    output_file: Option<String>,
}

impl GenerateCompleteArgs {
    pub fn invoke(self) -> eyre::Result<()> {
        let mut command = Args::command();

        let bin_name = self.bin_name;

        let mut output: Box<dyn Write> = if let Some(file) = self.output_file {
            Box::new(File::open(file)?)
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
        Ok(())
    }
}

shadow!(build_information);

#[derive(clap::Args, Debug)]
#[command(name = "information", about = "Print (debug) information")]
struct InformationArgs {}
impl InformationArgs {
    pub fn invoke(self) -> eyre::Result<()> {
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

        println!(
            "{}",
            ::zako_core::builtin::id::construct_builtins_typescript_export()
        );

        Ok(())
    }
}

fn setup_backtrace_env(enable_backtrace: bool) {
    #[cfg(debug_assertions)]
    let is_debug = true;
    #[cfg(not(debug_assertions))]
    let is_debug = false;

    let enable = is_debug || enable_backtrace;

    if std::env::var("RUST_SPANTRACE").is_err() {
        unsafe {
            if enable {
                std::env::set_var("RUST_SPANTRACE", "1");
            } else {
                std::env::set_var("RUST_SPANTRACE", "0");
            }
        }
    }

    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        unsafe {
            if enable {
                std::env::set_var("RUST_LIB_BACKTRACE", "full");
            } else {
                std::env::set_var("RUST_LIB_BACKTRACE", "1");
            }
        }
    }

    if std::env::var("RUST_BACKTRACE").is_err() {
        unsafe {
            if enable {
                std::env::set_var("RUST_BACKTRACE", "full");
            } else {
                std::env::set_var("RUST_BACKTRACE", "1");
            }
        }
    }

    if std::env::var("COLORBT_SHOW_HIDDEN").is_err() {
        unsafe {
            if enable {
                std::env::set_var("COLORBT_SHOW_HIDDEN", "1");
            } else {
                std::env::set_var("COLORBT_SHOW_HIDDEN", "0");
            }
        }
    }
}

fn inner_main() -> eyre::Result<()> {
    let (writer, handle) = TemporaryBufferedWriterMaker::new();

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder().build();

    let tracer = provider.tracer(env!("CARGO_BIN_NAME"));

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default().with(telemetry).with(
        HierarchicalLayer::new(2)
            .with_ansi(true)
            .with_indent_lines(true)
            .with_writer(writer),
    );

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = env::args_os();

    let _span = trace_span!(
        "program start",
        version = env!("CARGO_PKG_VERSION"),
        args = format!("{:?}", args)
    )
    .entered();

    let parse_args_span: tracing::span::EnteredSpan =
        trace_span!("prase arguments", args = format!("{:?}", args)).entered();

    let args = argfile::expand_args_from(args, argfile::parse_fromfile, argfile::PREFIX)?;

    trace!("argfile parsed {args:?}");

    let args = Args::parse_from(args);

    parse_args_span.exit();

    // 尘埃落定
    // 不是输出help/version，并且不是静默模式，倾泻而出
    //
    //《江城子·北邮校徽戏作》
    // 蓟门烟树郁苍苍，
    // 立邮邦，
    // 气昂扬。
    // 赫赫徽标，
    // 展翅正翱翔。
    // 本是传书通四海，
    // 观翼下，
    // 意味长。
    // 谁知学子话荒唐，
    // 指高墙，
    // 费思量。
    // 一点圆圆，
    // 直坠向中央。
    // 莫道宏图多壮志，
    // 看此势，
    // 是拉翔。
    if !args.silent {
        match handle.lock() {
            Ok(mut guard) => {
                guard.release()?;
            }
            Err(e) => {
                return Err(eyre::eyre!("failed to lock buffered writer: {}", e));
            }
        }
    } else {
        // 为了防止内存泄露，清空缓冲区并不再写入，静默成功
        match handle.lock() {
            Ok(mut guard) => {
                guard.silent();
            }
            Err(e) => {
                return Err(eyre::eyre!("failed to lock buffered writer: {}", e));
            }
        }
    }

    if let Some(dir) = args.chdir {
        let dir = PathBuf::from(dir);
        let dir = dir
            .canonicalize()
            .expect("failed to canonicalize directory");
        env::set_current_dir(&dir).expect("failed to change directory");
    }

    info!("working directory: {}", env::current_dir()?.display());

    args.color.write_global();
    setup_backtrace_env(args.backtrace);

    match ::colorchoice::ColorChoice::global() {
        ::colorchoice::ColorChoice::Auto
        | ::colorchoice::ColorChoice::AlwaysAnsi
        | ::colorchoice::ColorChoice::Always => {
            install_color_eyre_hook();
        }
        ::colorchoice::ColorChoice::Never => {}
    };

    return match args.command {
        SubCommands::Information(args) => args.invoke(),
        SubCommands::GenerateComplete(args) => args.invoke(),
        SubCommands::Make(args) => args.invoke(),
        SubCommands::ExportBuiltin(args) => args.invoke(),
        SubCommands::Bun(args) => run_bun(args.args),
    };
}

fn make_user_report_bug() {
    eprintln!(
        "{}",
        "Uncaught PANIC from zako. It is a bug, report it. Get in touch by `zako --help`"
            .red()
            .on_white()
    );
    eprintln!("  {}", "EXIT".red().bold());
}

/// This hook report panic to both user and tracing system
///
/// It should always be called when panic occurs
fn panic_hook(info: &std::panic::PanicHookInfo<'_>) {
    let payload = info.payload();

    let message = if let Some(s) = payload.downcast_ref::<&str>() {
        s
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.as_str()
    } else {
        "Unknown panic message"
    };

    let location = info
        .location()
        .map(|l| format!("at file `{}` line `{}`", l.file(), l.line()))
        .unwrap_or_default();

    let span_id = Span::current().id();

    eprintln!(
        "Panic occurred at `{}` (span {:?}:{}",
        location.red().bold(),
        span_id,
        message.red().bold(),
    );

    tracing::error!(
        panic.message = message,
        panic.location = location,
        panic.span_id = ?span_id,
        "Application panicked!"
    );

    make_user_report_bug();
}

/// Install color-eyre panic hook with original panic hook
///
/// This is optional, but [panic_hook] must install.
fn install_color_eyre_hook() {
    let eyre = Box::new(
        match color_eyre::config::HookBuilder::default()
            .display_env_section(true)
            .display_location_section(true)
            .try_into_hooks()
        {
            Ok(hooks) => hooks,
            Err(e) => {
                eprintln!(
                    "{}: {:?}",
                    "Failed to build color-eyre hook".red().bold(),
                    e
                );
                return;
            }
        },
    );

    _ = eyre.1.install().inspect_err(|err| {
        eprintln!(
            "{}: {:?}",
            "Failed to install color-eyre panic hook".red().bold(),
            err
        );
    });

    let eyre = eyre.0.into_panic_hook();

    let origin = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        eprintln!("{}", "----- color-eyre panic hook -----".red().bold());
        eyre(info);
        eprintln!("{}", "----- zako panic hook -----".red().bold());
        origin(info);
    }));
}

pub fn main() {
    let now = std::time::Instant::now();

    std::panic::set_hook(Box::new(panic_hook));

    let code = inner_main()
        .map(|_| exit_code::SUCCESS)
        .unwrap_or_else(|e| {
            eprintln!("{}: {:?}", "ERROR".red().bold(), e);
            eprintln!("  {}", "EXIT".red().bold());
            exit_code::FAILURE
        });

    let duration = now.elapsed();
    println!(
        "Elapsed {}{}{} {} == {}",
        duration.as_secs().green(),
        ".".white(),
        duration.subsec_nanos().yellow(),
        "seconds".cyan(),
        humantime::format_duration(duration).to_string().magenta()
    );

    ::std::process::exit(code);
}
