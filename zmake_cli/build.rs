use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Response;
use shadow_rs::BuildPattern;
use shadow_rs::ShadowBuilder;
use std::env;
use std::io::Write;

async fn download(url: String, destination: std::path::PathBuf) -> eyre::Result<()> {
    let response = reqwest::get(&url).await?;

    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .unwrap()
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
        target.write(&chunk);
        pb.set_position(downloaded);
    }
    pb.finish_with_message("Download complete!");

    Ok(())
}

async fn download_deno() -> eyre::Result<()> {
    let version = format!("v{}", "2.5.6");

    let base_url = format!(
        "https://github.com/denoland/deno/releases/download/{}",
        version
    );

    let zip_url = format!(
        "{}/deno-{}-{}-{}.zip",
        base_url,
        env::var("CARGO_CFG_TARGET_ARCH").unwrap(),
        env::var("CARGO_CFG_TARGET_VENDOR").unwrap(),
        if env::var("CARGO_CFG_TARGET_OS").unwrap().eq("macos") {
            "darwin"
        } else if env::var("CARGO_CFG_TARGET_OS").unwrap().eq("windows") {
            "windows-msvc"
        } else {
            "linux-gnu"
        }
    );
    let checksum_url = format!("{}.sha256sum", zip_url);

    let zip =
        std::path::PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set")).join("deno.zip");
    let bin = std::path::PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set")).join("deno");
    let checksum = std::path::PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"))
        .join("deno.sha256sum");

    if !std::fs::exists(&bin)? {
        download(zip_url, zip.clone()).await?;

        let zip = std::fs::File::open(zip).unwrap();

        let mut archive = zip::ZipArchive::new(zip).unwrap();

        archive.extract(env::var("OUT_DIR").expect("OUT_DIR not set"))?;
    }

    if !std::fs::exists(&checksum)? {
        download(checksum_url, checksum).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    color_eyre::install().unwrap_or_else(|_| println!("failed to install color-eyre"));

    download_deno().await.unwrap();

    ShadowBuilder::builder()
        .build_pattern(BuildPattern::RealTime)
        .build()
        .unwrap();
}
