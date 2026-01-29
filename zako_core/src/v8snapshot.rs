pub static PACKAGE_SNAPSHOT_FILE_NAME: &'static str = "package.bin";

#[cfg(not(feature = "v8snapshot"))]
pub static SNAPSHOT_OUT_DIR: &'static str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/", "zako.v8.snapshot");
#[cfg(feature = "v8snapshot")]
pub static SNAPSHOT_OUT_DIR: &'static str = "zako.v8.snapshot";

#[cfg(feature = "v8snapshot")]
pub static PACKAGE_SNAPSHOT: Option<&'static [u8]> = Some(include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/",
    "zako.v8.snapshot",
    "/",
    "package.bin"
)));
#[cfg(not(feature = "v8snapshot"))]
pub static PACKAGE_SNAPSHOT: Option<&'static [u8]> = None;
