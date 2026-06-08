/// libraw-rs example: read metadata from a RAW file.
///
/// Usage: cargo run --example simple -- path/to/file.CR3
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .expect("Usage: simple <raw-file>");

    let raw = libraw_rs::LibRaw::open(&path)?;

    // Read metadata
    let info = raw.image_info();
    println!("Camera:  {} {}", info.make(), info.model());
    println!("Software: {}", info.software());
    println!("Colors:   {}", info.colors());
    println!(
        "Filters:  {:#x} ({} Foveon)",
        info.filters(),
        if info.is_foveon() { "is" } else { "not" }
    );

    let sizes = raw.sizes();
    println!(
        "Size:     {}x{} (raw: {}x{})",
        sizes.width(),
        sizes.height(),
        sizes.raw_width(),
        sizes.raw_height()
    );

    let other = raw.image_other();
    println!("ISO:      {}", other.iso_speed());
    if other.shutter() > 0.0 {
        println!("Shutter:  1/{}s", (1.0 / other.shutter()) as i32);
    }
    println!("Aperture: f/{:.1}", other.aperture());
    println!("Focal:    {}mm", other.focal_len());

    let lens = raw.lens_info();
    if !lens.model().is_empty() {
        println!(
            "Lens:     {} {} ({:.0}-{:.0}mm)",
            lens.make(),
            lens.model(),
            lens.min_focal(),
            lens.max_focal()
        );
    }

    Ok(())
}
