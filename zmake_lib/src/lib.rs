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
pub mod sandbox;
mod local_cas;
mod make_builtin;
mod module_loader;
mod platform;
pub mod project;
pub mod project_resolver;
pub mod socket_address;
pub mod target;
mod tool;
mod transformer;
mod transport_server;
pub mod version_extractor;
pub mod path;
mod module_specifier;

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

    pub mod transport {
        tonic::include_proto!("zmake.v1.transport");
    }
}
