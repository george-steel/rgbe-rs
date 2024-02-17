pub use crate::types::*;

use image::{codecs::{hdr, png}, ImageDecoder, ImageEncoder, ImageError, ImageResult};
use std::{fs::File, io::{BufRead, BufReader, Read, Write}, path::Path};

pub fn decode_radiance<R:BufRead>(dec: hdr::HdrDecoder<R>) -> ImageResult<Box<[RGBE8]>> {
    let meta = dec.metadata();
    let size = (meta.width * meta.height) as usize;
    let mut out = bytemuck::allocation::zeroed_slice_box::<RGBE8>(size);
    dec.read_image_transform(|px| {
        RGBE8{r: px.c[0], g: px.c[1], b: px.c[2], e: px.e}
    }, &mut out)?;
    Ok(out)
}

pub fn decode_radiance_as_rgb9e5<R:BufRead>(dec: hdr::HdrDecoder<R>) -> ImageResult<Box<[RGB9E5]>> {
    let meta = dec.metadata();
    let size = (meta.width * meta.height) as usize;
    let mut out = bytemuck::allocation::zeroed_slice_box::<RGB9E5>(size);
    dec.read_image_transform(|px| {
        RGBE8{r: px.c[0], g: px.c[1], b: px.c[2], e: px.e}.repack_rgb9e5()
    }, &mut out)?;
    Ok(out)
}

pub fn load_radiance_file(path: &Path) -> ImageResult<(u32, u32, Box<[RGBE8]>)> {
    let file = File::open(path).map_err(ImageError::IoError)?;
    let decoder = hdr::HdrDecoder::new(BufReader::new(file))?;
    let meta = decoder.metadata();
    let data = decode_radiance(decoder)?;
    Ok((meta.width, meta.height, data))
}

pub fn decode_rgbe8_png<R:Read>(dec: png::PngDecoder<R>) -> ImageResult<Box<[RGBE8]>> {
    let (width, height) = dec.dimensions();
    let size = (width * height) as usize;
    let mut out = bytemuck::allocation::zeroed_slice_box::<RGBE8>(size);
    dec.read_image(bytemuck::cast_slice_mut(&mut out))?;
    Ok(out)
}

pub fn decode_rgbe8_png_as_rgb9e5<R:Read>(dec: png::PngDecoder<R>) -> ImageResult<Box<[RGB9E5]>> {
    let (width, height) = dec.dimensions();
    let size = (width * height) as usize;
    let mut orig = bytemuck::allocation::zeroed_slice_box::<RGBE8>(size);
    dec.read_image(bytemuck::cast_slice_mut(&mut orig))?;
    let out = orig.iter().copied().map(RGBE8::repack_rgb9e5).collect();
    Ok(out)
}

pub fn load_rgbe8_png_file(path: &Path) -> ImageResult<(u32, u32, Box<[RGBE8]>)> {
    let file = File::open(path).map_err(ImageError::IoError)?;
    let decoder = png::PngDecoder::new(file)?;
    let (width, height) = decoder.dimensions();
    let data = decode_rgbe8_png(decoder)?;
    Ok((width, height, data))
}

pub fn load_rgbe8_png_file_as_rgb9e5(path: &Path) -> ImageResult<(u32, u32, Box<[RGB9E5]>)> {
    let file = File::open(path).map_err(ImageError::IoError)?;
    let decoder = png::PngDecoder::new(file)?;
    let (width, height) = decoder.dimensions();
    let data = decode_rgbe8_png_as_rgb9e5(decoder)?;
    Ok((width, height, data))
}

pub fn encode_rgbe8_png<W: Write>(width: u32, height: u32, data: &[RGBE8], out: W) -> ImageResult<()> {
    let encoder = png::PngEncoder::new_with_quality(out, png::CompressionType::Best, png::FilterType::Adaptive);
    encoder.write_image(bytemuck::cast_slice(data), width, height, image::ColorType::Rgba8)?;
    Ok(())
}

pub fn save_rgbe8_png_file(path: &Path, width: u32, height: u32, data: &[RGBE8]) -> ImageResult<()> {
    let file = File::create(path).map_err(ImageError::IoError)?;
    encode_rgbe8_png(width, height, data, file)
}

