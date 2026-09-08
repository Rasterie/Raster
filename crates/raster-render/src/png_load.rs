use std::fmt;
use std::path::Path;

/// Decoded pixel data, ready to upload as a texture.
#[derive(Clone)]
pub struct Image {
    /// RGBA8, row-major, no padding.
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Shows the dimensions rather than the pixels: a debug print of a megabyte of
/// bytes is unreadable and hides whatever it was meant to reveal.
impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("bytes", &self.pixels.len())
            .finish()
    }
}

/// Why an image could not be read.
#[derive(Debug)]
pub enum ImageError {
    Io(std::io::Error),
    /// The file is not a valid PNG, or uses a feature the decoder rejects.
    Decode(png::DecodingError),
    /// A colour format the engine does not convert.
    UnsupportedFormat {
        colour: png::ColorType,
        depth: png::BitDepth,
    },
    /// A dimension is zero, or the image is larger than a texture may be.
    BadSize {
        width: u32,
        height: u32,
    },
}

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "could not read the file: {e}"),
            Self::Decode(e) => write!(f, "not a valid PNG: {e}"),
            Self::UnsupportedFormat { colour, depth } => write!(
                f,
                "unsupported PNG format: {colour:?} at {depth:?}; \
                 the engine reads 8-bit greyscale and RGB/RGBA, with or without alpha"
            ),
            Self::BadSize { width, height } => {
                write!(f, "an image cannot be {width}x{height}")
            }
        }
    }
}

impl std::error::Error for ImageError {}

impl From<std::io::Error> for ImageError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<png::DecodingError> for ImageError {
    fn from(e: png::DecodingError) -> Self {
        Self::Decode(e)
    }
}

impl Image {
    /// The largest texture the engine will build.
    ///
    /// Matches the lowest guaranteed GPU limit. Refusing here gives a clear
    /// message rather than a validation error from the driver.
    pub const MAX_DIMENSION: u32 = 8192;

    /// Reads a PNG from disk.
    ///
    /// # Errors
    ///
    /// Fails if the file cannot be read, is not a PNG, uses a colour format
    /// the engine does not convert, or has an unusable size.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ImageError> {
        let file = std::fs::File::open(path)?;
        Self::decode(std::io::BufReader::new(file))
    }

    /// Decodes a PNG from a buffered, seekable source — a file, an embedded
    /// byte slice, an archive entry.
    ///
    /// `Seek` is required by the decoder, which reads chunk headers before
    /// deciding what to do with them. A `&[u8]` and a `Cursor` both satisfy it.
    ///
    /// # Errors
    ///
    /// As [`Image::load`], minus the file access.
    pub fn decode(source: impl std::io::BufRead + std::io::Seek) -> Result<Self, ImageError> {
        let decoder = png::Decoder::new(source);
        let mut reader = decoder.read_info()?;

        let mut buffer = vec![0; reader.output_buffer_size().unwrap_or(0)];
        let info = reader.next_frame(&mut buffer)?;

        let (width, height) = (info.width, info.height);
        if width == 0 || height == 0 || width > Self::MAX_DIMENSION || height > Self::MAX_DIMENSION
        {
            return Err(ImageError::BadSize { width, height });
        }

        buffer.truncate(info.buffer_size());
        let pixels = to_rgba(&buffer, info.color_type, info.bit_depth, width, height)?;

        Ok(Self {
            pixels,
            width,
            height,
        })
    }

    /// The size, for building a sprite that draws the whole image.
    #[must_use]
    pub fn size(&self) -> raster_math::Vec2 {
        raster_math::Vec2::new(self.width as f32, self.height as f32)
    }
}

/// Converts decoded pixels to RGBA8.
///
/// The GPU takes one format; a PNG may be greyscale, indexed, or RGB. Rather
/// than a texture format per variant, everything becomes RGBA here — the cost
/// is paid once at load, not per frame.
fn to_rgba(
    data: &[u8],
    colour: png::ColorType,
    depth: png::BitDepth,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, ImageError> {
    // Le decodeur convertit deja les profondeurs inferieures a 8 bits et les
    // palettes en composantes 8 bits ; il ne reste que la disposition des
    // canaux a uniformiser.
    if depth != png::BitDepth::Eight && depth != png::BitDepth::Sixteen {
        return Err(ImageError::UnsupportedFormat { colour, depth });
    }

    let count = (width as usize) * (height as usize);
    let mut rgba = Vec::with_capacity(count * 4);

    // Un PNG 16 bits arrive en octets de poids fort en premier : on ne garde
    // que celui-la, le pixel art n'ayant aucun usage de la precision
    // supplementaire.
    let stride = if depth == png::BitDepth::Sixteen {
        2
    } else {
        1
    };

    match colour {
        png::ColorType::Rgba => {
            for i in 0..count {
                let p = i * 4 * stride;
                rgba.extend_from_slice(&[
                    data[p],
                    data[p + stride],
                    data[p + 2 * stride],
                    data[p + 3 * stride],
                ]);
            }
        }
        png::ColorType::Rgb => {
            for i in 0..count {
                let p = i * 3 * stride;
                rgba.extend_from_slice(&[data[p], data[p + stride], data[p + 2 * stride], 255]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for i in 0..count {
                let p = i * 2 * stride;
                let g = data[p];
                rgba.extend_from_slice(&[g, g, g, data[p + stride]]);
            }
        }
        png::ColorType::Grayscale => {
            for i in 0..count {
                let g = data[i * stride];
                rgba.extend_from_slice(&[g, g, g, 255]);
            }
        }
        // Le decodeur developpe les palettes en RGB ou RGBA : ce cas ne
        // devrait pas se presenter, mais mieux vaut le dire que le supposer.
        png::ColorType::Indexed => {
            return Err(ImageError::UnsupportedFormat { colour, depth });
        }
    }

    Ok(rgba)
}
