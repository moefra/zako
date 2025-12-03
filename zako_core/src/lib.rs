pub mod access_control;
pub mod build_constants;
pub mod builtin;
mod cas;
mod cas_server;
pub mod configuration;
mod digest;
pub mod engine;
mod error;
mod extension;
pub mod file_finder;
pub mod fs;
pub mod id;
mod local_cas;
mod make_builtin;
mod module_loader;
mod module_specifier;
pub mod path;
pub mod pattern;
mod platform;
pub mod project;
pub mod project_resolver;
pub mod sandbox;
pub mod socket_address;
pub mod target;
mod tool;
mod transformer;
mod transport_server;
pub mod version_extractor;

pub static PROJECT_FILE_NAME: &str = "ZProject.ts";
pub static BUILD_FILE_NAME:&str = "ZBUILD.ts";
pub static RULE_FILE_SUFFIX:&str = ".zako.ts";
pub static SCRIPT_FILE_SUFFIX:&str = ".zscript.ts";

pub mod proto {
    pub mod digest {
        tonic::include_proto!("zako.v1.digest");
    }

    pub mod fs {
        tonic::include_proto!("zako.v1.fs");
    }

    pub mod net {
        tonic::include_proto!("zako.v1.net");
    }

    pub mod cas {
        tonic::include_proto!("zako.v1.cas");
    }

    pub mod transport {
        tonic::include_proto!("zako.v1.transport");
    }
}
