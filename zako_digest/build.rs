use std::io::Result;

fn main() -> Result<()> {
    tonic_prost_build::configure()
        .compile_protos(&["src/protobuf/digest.proto"], &["src/protobuf/"])?;
    Ok(())
}
