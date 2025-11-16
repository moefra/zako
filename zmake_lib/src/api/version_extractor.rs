use std::process::{Command, Stdio};
use semver::Version;
use tracing::{span, trace, trace_span};
use tracing_core::Level;

fn extract_from_string(string:String) -> Option<Version>{
    let _span = trace_span!("try extract program version", extract_from = string).entered();

    for part in string.trim().split_whitespace(){
        if let Ok(version) = Version::parse(part){
            return Some(version)
        }
        else if let Ok(version) = lenient_semver::parse(part){
            return Some(version)
        }
    }
    None
}

fn extract_from_command(program:&str,argument:&str) -> Option<Version>{
    let _span = trace_span!("try get program version", program,argument).entered();

    if let Ok(result) = Command::new(program)
        .env_clear()
        .stdin(Stdio::null())
        .arg(argument)
        .output() {
        if let Ok(output) = String::from_utf8(result.stdout) {
            match extract_from_string(output) {
                Some(version) => Some(version),
                None => if let Ok(output) = String::from_utf8(result.stderr) {
                    extract_from_string(output)
                } else {
                    None
                }
            }
        }
        else{
            None
        }
    }
    else{
        None
    }
}

pub fn extract_version(program_file:String) -> Option<Version>{
    let _span = trace_span!("try get program version", program_file).entered();

    if let Some(version) = extract_from_command(&program_file,"--version")
    {
        trace!("Extract {version} from {program_file} with argument `--version`");
        Some(version)
    }
    else if let Some(version) = extract_from_command(&program_file,"-V"){
        trace!("Extract {version} from {program_file} with argument `-V`");
        Some(version)
    }
    else {
        None
    }
}
