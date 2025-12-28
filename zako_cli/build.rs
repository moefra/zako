use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use shadow_rs::BuildPattern;
use shadow_rs::ShadowBuilder;
use std::arch;
use std::env;
use std::hash;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::PathBuf;

async fn download(url: String, destination: std::path::PathBuf) -> eyre::Result<()> {
    println!("download file `{}`", &url);

    let response = reqwest::get(&url).await?;

    let total_size = response.content_length().unwrap_or(0);

    // cargo build -vv to see this progress bar
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
            .progress_chars("#>-"));
    pb.set_message(format!(
        "Downloading file {}",
        destination.to_string_lossy()
    ));

    let mut downloaded = 0;
    let mut stream = response.bytes_stream();
    let mut target = std::fs::File::create(destination)?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        downloaded += chunk.len() as u64;
        target.write_all(&chunk)?;
        pb.set_position(downloaded);
    }
    pb.finish_with_message("Download complete!");
    println!(
        "downloaded finish, downloaded size {} mb",
        downloaded / 1024 / 1024
    );

    Ok(())
}

async fn download_and_extract(
    base_url: &str,
    version: &str,
    name: &str,
    zip_file_name: &str,
) -> Result<(), eyre::Error> {
    let base_url = format!("{}/{}", base_url.trim_end_matches("/"), version);
    let zip_url = format!("{}/{}", base_url, zip_file_name);

    let zip = std::path::PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"))
        .join(format!("{}.zip", name));
    let bin = std::path::PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set")).join(name);

    let force_redownload: bool = env::var("ZAKO_FORCE_REDOWNLOAD")
        .unwrap_or("false".into())
        .parse()?;

    if force_redownload || (!std::fs::exists(&bin)?) {
        download(zip_url, zip.clone()).await?;

        let zip = std::fs::File::open(zip).unwrap();

        let mut archive = zip::ZipArchive::new(zip)?;

        if archive.len() == 1 {
            let mut file = archive.by_index(0)?;
            let mut bin = std::fs::File::create(bin)?;
            io::copy(&mut file, &mut bin)?;
        } else if archive.len() == 2 {
            let first_is_file = archive.by_index(0)?.is_file();
            let seconds_is_file = archive.by_index(1)?.is_file();
            if first_is_file && seconds_is_file {
                return Err(eyre::eyre!(
                    "The item counts of name.zip archive 2,but there were two dir or two file(expect one dir and one file)"
                ));
            }

            let maybe_file = archive.by_index(0)?;
            let mut file = if maybe_file.is_file() {
                maybe_file
            } else {
                drop(maybe_file);
                archive.by_index(1)?
            };
            let mut bin = std::fs::File::create(bin)?;
            io::copy(&mut file, &mut bin)?;
        } else {
            return Err(eyre::eyre!(
                "The item counts of {}.zip archive must be 1 or 2",
                name
            ));
        }
    }

    Ok(())
}

async fn zstd_compress(input: &str, output: &str) -> eyre::Result<()> {
    let mut input_file = std::fs::File::open(input)?;
    let mut output_file = std::fs::File::create(output)?;

    let mut encoder = zstd::stream::Encoder::new(&mut output_file, 22)?;
    io::copy(&mut input_file, &mut encoder)?;
    encoder.finish()?;

    Ok(())
}

async fn download_bun() -> eyre::Result<()> {
    download_and_extract(
        "https://github.com/oven-sh/bun/releases/download/",
        "bun-v1.3.3",
        "bun",
        &format!(
            "bun-{}-{}.zip",
            if env::var("CARGO_CFG_TARGET_OS").unwrap().eq("macos") {
                "darwin"
            } else if env::var("CARGO_CFG_TARGET_OS").unwrap().eq("windows") {
                "windows"
            } else {
                "linux"
            },
            env::var("CARGO_CFG_TARGET_ARCH").unwrap(),
        ),
    )
    .await?;
    if !std::fs::exists(
        std::path::PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set")).join("bun.zst"),
    )? {
        zstd_compress(
            &std::path::PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"))
                .join("bun")
                .to_string_lossy(),
            &std::path::PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"))
                .join("bun.zst")
                .to_string_lossy(),
        )
        .await?
    }
    Ok(())
}

async fn download_deno() -> eyre::Result<()> {
    download_and_extract(
        "https://github.com/denoland/deno/releases/download/",
        "v2.5.6",
        "deno",
        &format!(
            "deno-{}-{}-{}.zip",
            env::var("CARGO_CFG_TARGET_ARCH").unwrap(),
            env::var("CARGO_CFG_TARGET_VENDOR").unwrap(),
            if env::var("CARGO_CFG_TARGET_OS").unwrap().eq("macos") {
                "darwin"
            } else if env::var("CARGO_CFG_TARGET_OS").unwrap().eq("windows") {
                "windows-msvc"
            } else {
                "linux-gnu"
            }
        ),
    )
    .await
}

#[tokio::main]
async fn main() {
    color_eyre::install().unwrap_or_else(|_| println!("failed to install color-eyre"));

    // download_deno().await.unwrap();
    download_bun().await.unwrap();

    ShadowBuilder::builder()
        .build_pattern(BuildPattern::RealTime)
        .build()
        .unwrap();
}
