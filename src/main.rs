#![allow(unused_parens)]

use clap::Parser;
use cuda_interface::args::Args;
use cuda_interface::buffer::Buffer;
use cuda_interface::kernel::Kernel;
use cuda_interface::stream::Stream;
use std::fs::{read_to_string, File};
use std::io::BufWriter;
use std::os::raw::c_void;
use std::path::Path;

#[link(name = "kernel", kind = "static")]
extern "C" {
    fn julia() -> c_void;
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ClapArgs {
    /// Width in pixels.
    #[arg(short, long, default_value_t = 1920)]
    width: u32,

    /// Height in pixels.
    #[arg(short = 'H', long, default_value_t = 1080)]
    height: u32,

    /// Number of iterations.
    #[arg(short, long, default_value_t = 500)]
    iterations: u32,

    /// Real part of seed.
    #[arg(short, default_value_t = 0.285)]
    x: f32,

    /// Imaginary part of seed.
    #[arg(short, default_value_t = 0.01)]
    y: f32,

    /// Y-coordinate of top.
    #[arg(short, long, default_value_t = 1.2)]
    top: f32,

    /// Y-coordinate of bottom.
    #[arg(short, long, default_value_t = -1.2)]
    bottom: f32,

    /// X-coordinate of left.
    #[arg(short, long, default_value_t = -2.1)]
    left: f32,

    /// X-coordinate of right.
    #[arg(short, long, default_value_t = 2.1)]
    right: f32,

    /// Path to an 8-bit RGB color map of length 256.
    #[arg(short, long, default_value_t = ("./resources/default.cmap".to_string()))]
    color_map: String,

    ///
    #[arg(short, long, default_value_t = ("output.png".to_string()))]
    output: String,
}

fn read_color_map(path: &Path) -> Buffer<u8> {
    let mut color_map: Buffer<u8> = Buffer::new(3 * 256).unwrap();

    for (i, line) in read_to_string(path).unwrap().lines().enumerate() {
        let mut line = line.split(' ');
        let r = line.next().unwrap().parse::<u8>().unwrap();
        let g = line.next().unwrap().parse::<u8>().unwrap();
        let b = line.next().unwrap().parse::<u8>().unwrap();
        color_map.write(i * 3, &[r, g, b]).unwrap();
    }

    color_map
}

fn write_png(image: &[u8], output: &Path, width: u32, height: u32) {
    let file = File::create(output).unwrap();
    let w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(image).unwrap();
}

fn main() {
    let clap_args = ClapArgs::parse();

    let kernel = Kernel::new(julia);
    let mut stream = Stream::new().unwrap();

    let color_map_path = Path::new(&clap_args.color_map);
    let mut color_map = read_color_map(color_map_path);
    let mut image: Buffer<u8> =
        Buffer::new(3 * clap_args.width as usize * clap_args.height as usize).unwrap();

    let mut args = Args::default();
    args.add_arg(&clap_args.width);
    args.add_arg(&clap_args.height);
    args.add_arg(&clap_args.x);
    args.add_arg(&clap_args.y);
    args.add_arg(&clap_args.iterations);
    args.add_arg(&clap_args.top);
    args.add_arg(&clap_args.bottom);
    args.add_arg(&clap_args.left);
    args.add_arg(&clap_args.right);
    args.add_arg(&mut image);
    args.add_arg(&mut color_map);

    let grid_dim = (clap_args.width / 32, clap_args.height, 1);
    let block_dim = (32, 1, 1);

    stream
        .launch(&kernel, grid_dim, block_dim, &args, 0)
        .unwrap();
    stream.wait().unwrap();

    let image = image.read_all().unwrap();
    let output = Path::new(&clap_args.output);
    write_png(&image, output, clap_args.width, clap_args.height);
}
