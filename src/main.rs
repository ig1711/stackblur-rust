use png;
use stackblur::stackblur;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::SystemTime;

use clap::Parser;

/// Blur png images
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Source
    #[clap(short, long)]
    source: String,

    /// Destination
    #[clap(short, long)]
    destination: String,

    /// Radius
    #[clap(short, long, default_value_t = 21)]
    radius: u8,
}

fn main() {
    let args = Args::parse();
    let decoder = png::Decoder::new(File::open(&args.source).expect("Failed to open source file"));
    let mut reader = decoder.read_info().expect("Failed to decode the image");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("Failed to read data");
    let channels = match info.color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::Rgb => 3,
        png::ColorType::Indexed => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgba => 4,
    };

    let bytes = &buf[..info.buffer_size()];

    let stime = SystemTime::now();
    let hmm = stackblur::blur(
        bytes,
        info.width as usize,
        info.height as usize,
        channels,
        args.radius as usize,
    );
    let elapsed = stime.elapsed().expect("Failed to measure time");
    println!("Generated in: {:?} milliseconds", elapsed.as_millis());

    let path = Path::new(&args.destination);
    let file = File::create(path).expect("Failed to create destination file");
    let ref mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, info.width, info.height);
    encoder.set_color(info.color_type);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_trns(vec![0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455));
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
    let source_chromaticities = png::SourceChromaticities::new(
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000),
    );
    encoder.set_source_chromaticities(source_chromaticities);
    let mut writer = encoder
        .write_header()
        .expect("Failed to encode (write header)");

    writer.write_image_data(&hmm[..]).expect("Failed to encode");
}
