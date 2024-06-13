//!  A module for encoding/decoding.

// from rust
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

// from external crate
use gif;
use piston_image::GenericImage;
use png;

// from local crate
use error::{RasterError, RasterResult};
use Image;
use ImageFormat;

// Decode GIF
pub fn decode_gif(image_file: &File) -> RasterResult<Image> {
    let mut decoder = gif::Decoder::new(image_file);

    // Configure the decoder such that it will expand the image to RGBA.
    gif::SetParameter::set(&mut decoder, gif::ColorOutput::RGBA);

    // Read the file header
    let mut reader = decoder.read_info()?;

    // Read frame 1.
    // TODO: Work on all frames
    if let Some(_) = reader.next_frame_info()? {
        let mut bytes = vec![0; reader.buffer_size()];
        reader.read_into_buffer(&mut bytes)?;
        Ok(Image {
            width: reader.width() as i32,
            height: reader.height() as i32,
            bytes: bytes,
        })
    } else {
        Err(RasterError::Decode(
            ImageFormat::Gif,
            "Error getting frame info".to_string(),
        ))
    }
}

// Encode GIF
pub fn encode_gif(image: &Image, path: &Path) -> RasterResult<()> {
    // Open the file with basic error check
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let frame = gif::Frame::from_rgba(
        image.width as u16,
        image.height as u16,
        &mut image.bytes.clone(),
    ); // TODO: Perf issue?
    let mut encoder = gif::Encoder::new(writer, frame.width, frame.height, &[])?;
    encoder.write_frame(&frame).map_err(RasterError::Io)?;
    Ok(())
}

// Decode PNG
pub fn decode_png(image_file: &File) -> RasterResult<Image> {
    let decoder = png::Decoder::new(image_file);
    let (info, mut reader) = decoder.read_info()?;
    let mut bytes = vec![0; info.buffer_size()];

    reader.next_frame(&mut bytes)?;

    if info.color_type == png::ColorType::RGB {
        // Applies only to RGB

        let mut insert_count = 0;
        let len = (info.width * info.height) as usize;
        for i in 0..len {
            // TODO: This is slow!
            let insert_pos = 3 * (i + 1) + insert_count;
            bytes.insert(insert_pos, 255);
            insert_count += 1;
        }
    } //  TODO other ::ColorType
    Ok(Image {
        width: info.width as i32,
        height: info.height as i32,
        bytes: bytes,
    })
}

// Encode PNG
pub fn encode_png(image: &Image, path: &Path) -> RasterResult<()> {
    // Open the file with basic error check
    let file = File::create(path)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, image.width as u32, image.height as u32);
    png::HasParameters::set(&mut encoder, png::ColorType::RGBA);
    png::HasParameters::set(&mut encoder, png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    Ok(writer.write_image_data(&image.bytes)?)
}

pub fn decode_file<P: AsRef<Path>>(image_file: P, format: ImageFormat) -> RasterResult<Image> {
    match format {
        ImageFormat::Gif => {
            let file = File::open(image_file)?;
            Ok(decode_gif(&file)?)
        }
        ImageFormat::Jpeg => {
            let src = piston_image::open(image_file)?;
            let (w, h) = src.dimensions();
            let mut bytes = Vec::with_capacity((w * h) as usize * 4);
            for y in 0..h {
                for x in 0..w {
                    let p = src.get_pixel(x, y);
                    bytes.extend_from_slice(&p.data[0..4]);
                }
            }
            Ok(Image {
                width: w as i32,
                height: h as i32,
                bytes: bytes,
            })
        }
        ImageFormat::Png => {
            let file = File::open(image_file)?;
            Ok(decode_png(&file)?)
        }
    }
}
