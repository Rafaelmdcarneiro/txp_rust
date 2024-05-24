use ::image::dxt::{DXTVariant, DxtDecoder};
use ::image::*;

use super::*;

use std::path::Path;

impl<'a> Mipmap<'a> {
    pub fn to_dxt_decoder(&self) -> Option<Result<DxtDecoder<&[u8]>, ImageError>> {
        use TextureFormat::*;
        let format = match self.format {
            DXT1 | DXT1a => Some(DXTVariant::DXT1),
            DXT3 => Some(DXTVariant::DXT3),
            DXT5 => Some(DXTVariant::DXT5),
            _ => return None,
        }?;
        Some(DxtDecoder::new(&self.data, self.width, self.height, format))
    }

    pub fn to_rgb(&self) -> Option<ImageBuffer<Rgb<u8>, &[u8]>> {
        use TextureFormat::*;
        match self.format {
            RGB => ImageBuffer::from_raw(self.width, self.height, &self.data),
            _ => None,
        }
    }

    pub fn to_rgba(&self) -> Option<ImageBuffer<Rgba<u8>, &[u8]>> {
        use TextureFormat::*;
        match self.format {
            RGBA => ImageBuffer::from_raw(self.width, self.height, &self.data),
            _ => None,
        }
    }

    pub fn to_luma(&self) -> Option<ImageBuffer<Luma<u8>, &[u8]>> {
        use TextureFormat::*;
        match self.format {
            L8 => ImageBuffer::from_raw(self.width, self.height, &self.data),
            _ => None,
        }
    }

    pub fn to_luma_alpha(&self) -> Option<ImageBuffer<LumaA<u8>, &[u8]>> {
        match self.format {
            L8A8 => ImageBuffer::from_raw(self.width, self.height, &self.data),
            _ => None,
        }
    }

    pub fn to_dynamic_image(self) -> Option<DynamicImage> {
        use TextureFormat::*;
        match self.format {
            RGB => ImageBuffer::from_raw(self.width, self.height, self.data.into_owned())
                .map(DynamicImage::ImageRgb8),
            RGBA => ImageBuffer::from_raw(self.width, self.height, self.data.into_owned())
                .map(DynamicImage::ImageRgba8),
            L8 => ImageBuffer::from_raw(self.width, self.height, self.data.into_owned())
                .map(DynamicImage::ImageLuma8),
            L8A8 => ImageBuffer::from_raw(self.width, self.height, self.data.into_owned())
                .map(DynamicImage::ImageLumaA8),
            DXT1 | DXT1a | DXT3 | DXT5 => {
                let dec = self.to_dxt_decoder()?.ok()?;
                DynamicImage::from_decoder(dec).ok()
            }
            _ => return None,
        }
    }

    pub fn save<Q>(&self, path: Q) -> Option<ImageResult<()>>
    where
        Q: AsRef<Path>,
    {
        use TextureFormat::*;
        Some(match self.format {
            RGB => self.to_rgb()?.save(path),
            RGBA => self.to_rgba()?.save(path),
            L8 => self.to_luma()?.save(path),
            L8A8 => self.to_luma_alpha()?.save(path),
            DXT1 | DXT1a | DXT3 | DXT5 => {
                let dec = self.to_dxt_decoder()?.ok()?;
                let image = DynamicImage::from_decoder(dec).ok()?;
                image.save(path)
            }
            _ => return None,
        })
    }
}
