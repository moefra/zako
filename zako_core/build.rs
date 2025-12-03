use std::io::Result;

fn main() -> Result<()> {
    tonic_prost_build::configure().compile_protos(
        &[
            "src/proto/digest.proto",
            "src/proto/fs.proto",
            "src/proto/net.proto",
            "src/proto/cas.proto",
            "src/proto/transport.proto",
        ],
        &["src/proto/"],
    )?;
    Ok(())
}
