use ::std::io::Cursor;
use base64::{Engine as _, engine::general_purpose};
use std::env;
use std::fs;
use std::io::{self, Write, stdout};
use std::path::Path;

fn main() -> eyre::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        return Err(eyre::eyre!(
            "Usage: {} <input_image_path> <output_kitty_graphics_protocol_txt_path>",
            args[0]
        ));
    }

    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);

    let input = image::open(input_path)?;

    let png_buffer = {
        let png_buffer = vec![];

        let mut cursor = Cursor::new(png_buffer);

        input.write_to(&mut cursor, image::ImageFormat::Png)?;

        cursor.into_inner()
    };

    let sequence = render_png(png_buffer.as_slice());

    // test write
    {
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(format!("input image file {:?}:\n", input_path).as_bytes())?;
        stdout.write_all(sequence.as_bytes())?;
        stdout.write_all(b"\n")?;
        stdout.write_all(
            format!(
                "write to file {:?} as kitty graphics protocol\n",
                output_path
            )
            .as_bytes(),
        )?;
        stdout.flush()?;
    }

    fs::write(output_path, sequence)?;

    Ok(())
}

/// Render a PNG image to a string using kitty graphics protocol.
///
/// This version returns a [String] instead of printing to stdout.
pub fn render_png(png: &[u8]) -> String {
    use std::fmt::Write as _;

    let base64_data = general_purpose::STANDARD.encode(png);
    let chunks = base64_data.as_bytes().chunks(2048);
    let total_chunks = chunks.len();

    let mut res = String::new();

    for (i, chunk) in chunks.enumerate() {
        let is_last = i == total_chunks - 1;
        let is_first = i == 0;
        let m = if is_last { 0 } else { 1 };

        if is_first {
            // First chunk: a=T (transmit and display), f=100 (PNG)
            write!(res, "\x1b_Ga=T,f=100,m={};", m).expect("String write failed");
        } else {
            // Subsequent chunks: only m is required
            write!(res, "\x1b_Gm={};", m).expect("String write failed");
        }

        res.push_str(std::str::from_utf8(chunk).expect("Base64 is valid UTF-8"));
        res.push_str("\x1b\\");
    }

    res
}
