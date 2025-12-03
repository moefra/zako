use semver::Version;
use std::process::{Command, Stdio};
use tracing::{trace, trace_span};

fn extract_from_string(string: &str) -> Option<Version> {
    for part in string.trim().split_whitespace() {
        if let Ok(version) = Version::parse(part) {
            return Some(version);
        } else if let Ok(version) = lenient_semver::parse(part) {
            return Some(version);
        }
    }
    None
}

fn extract_from_command(program: &str, argument: &str) -> Option<Version> {
    match Command::new(program)
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .stdin(Stdio::null())
        .arg(argument)
        .output()
    {
        Ok(result) => {
            let from = String::from_utf8_lossy(&result.stdout);

            extract_from_string(&from).or_else(|| {
                let from = String::from_utf8_lossy(&result.stderr);
                extract_from_string(&from)
            })
        }
        Err(err) => {
            trace!("failed to run program when extract version:{}", err);
            None
        }
    }
}

pub fn extract_version(program_file: &str) -> Option<Version> {
    let _span = trace_span!("try get program version", program_file).entered();

    const VERSION_ARGS: &[&str] = &["--version", "-V"];

    for arg in VERSION_ARGS {
        if let Some(version) = extract_from_command(program_file, arg) {
            trace!(
                "found version {} for '{}' using argument '{}'",
                version, program_file, arg
            );
            return Some(version);
        } else {
            trace!(
                "failed to extract version from '{}' using argument '{}'",
                program_file, arg
            );
        }
    }

    None
}
