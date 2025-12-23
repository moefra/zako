use std::io::Result;

fn main() -> Result<()> {
    tonic_prost_build::configure()
        .extern_path(".zako.v1.digest", "::zako_digest::protobuf")
        .compile_protos(
            &[
                "src/protobuf/fs.proto",
                "src/protobuf/net.proto",
                "src/protobuf/cas.proto",
                "src/protobuf/transport.proto",
                "src/protobuf/range.proto",
            ],
            &["src/protobuf/", "./../zako_digest/src/protobuf/"],
        )?;
    Ok(())
}
