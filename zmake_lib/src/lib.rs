pub mod Isolate;
pub mod access_control;
pub mod build_constants;
pub mod builtin;
pub mod configuration;
mod digest;
pub mod engine;
mod error;
pub mod file_finder;
pub mod fs;
pub mod id;
mod make_builtin;
pub mod project;
pub mod project_resolver;
pub mod target;
mod tool;
pub mod version_extractor;
mod cas;
mod local_cas;

pub mod proto {
    pub mod digest {
        tonic::include_proto!("zmake.v1.digest");
    }

    pub mod fs {
        tonic::include_proto!("zmake.v1.fs");
    }

    pub mod net {
        tonic::include_proto!("zmake.v1.net");
    }

    pub mod cas {
        tonic::include_proto!("zmake.v1.cas");
    }
}
