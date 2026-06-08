/// libraw-rs example: extract JPEG preview thumbnail from a RAW file.
///
/// Usage: cargo run --example extract_thumb -- <input.RAW> [output.jpg]
///
/// If output path is omitted, the thumbnail is saved alongside the input
/// with "_thumb.jpg" appended.
use std::env;
use std::path::PathBuf;

use libraw_rs::LibRaw;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: extract_thumb <input.RAW> [output.jpg]");
        std::process::exit(1);
    }

    let input = &args[1];
    let output = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        let mut out = PathBuf::from(input);
        out.set_extension("thumb.jpg");
        out
    };

    let mut raw = LibRaw::open(input)?;

    // Unpack only the thumbnail (much faster than full unpack)
    raw.unpack_thumb()?;

    // Check what kind of thumbnail we got
    if raw.is_jpeg_thumb() {
        println!("JPEG thumbnail found, extracting...");
    } else {
        eprintln!("Warning: thumbnail is not JPEG format, saving raw data");
    }

    // Produce the in-memory thumbnail
    let thumb = raw.make_mem_thumb()?;

    println!("Thumbnail type: {:?}", thumb.image_type());
    println!("Data size:      {} bytes", thumb.data().len());
    println!("Dimensions:     {}×{}", thumb.width(), thumb.height());
    println!("Colors:         {}", thumb.colors());
    println!("Bits/sample:    {}", thumb.bits());

    thumb.save(&output)?;
    println!("Saved to:       {}", output.display());

    Ok(())
}
