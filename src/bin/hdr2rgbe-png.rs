use image::{ImageError, ImageResult};
use std::{env, io, path};

pub fn main() -> ImageResult<()> {
    let args: Box<[String]> = env::args().collect();
    if args.len() < 2 {
        return Err(ImageError::IoError(io::Error::new(io::ErrorKind::InvalidInput, "A filename is required")));
    }
    let path = path::Path::new(&args[1]);
    let (width, height, data) = rgbe::load_radiance_file(path)?;
    let outpath = path.with_extension("rgbe.png");
    rgbe::save_rgbe8_png_file(&outpath, width, height, &data)
}