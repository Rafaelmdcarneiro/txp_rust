#[cfg(feature = "image")]
use ::image::{Bgr, Bgra, DynamicImage, ImageBuffer, Rgb, Rgba};
use dcv_color_primitives::*;
use tracing::{debug, instrument, trace};

use super::*;

const NV12: ImageFormat = ImageFormat {
    pixel_format: PixelFormat::Nv12,
    color_space: ColorSpace::Bt709,
    num_planes: 2,
};

const BGRA: ImageFormat = ImageFormat {
    pixel_format: PixelFormat::Bgra,
    color_space: ColorSpace::Lrgb,
    num_planes: 1,
};

impl Texture<'_> {
    pub fn is_yuv(&self) -> bool {
        self.subtextures.len() == 1
            && self
                .subtextures
                .get(0)
                .map(Subtexture::is_yuv)
                .unwrap_or_default()
    }
}

impl Subtexture<'_> {
    pub fn is_yuv(&self) -> bool {
        self.mipmaps.len() == 2 && self.mipmaps.iter().all(|d| d.format == TextureFormat::ATI2)
    }

    #[tracing::instrument(skip(self))]
    pub fn yuv_to_bgra(&self) -> Result<Vec<u8>, ErrorKind> {
        dcv_color_primitives::initialize();

        let ay = &self.mipmaps[0];
        let uv = &self.mipmaps[1];

        let src_sizes: &mut [usize] = &mut [0usize; 2];
        get_buffers_size(ay.width, ay.height, &NV12, None, src_sizes)?;

        debug!("{}x{}", ay.width, ay.height);
        debug!("src: {}", ay.data.len());
        trace!(?src_sizes);

        let src_y: Vec<_> = vec![0u8; src_sizes[0]];
        let src_uv: Vec<_> = vec![0u8; src_sizes[1]];
        let src_buffers: &[&[u8]] = &[&src_y[..], &src_uv[..]];

        let mut y_data: Vec<u8> = Vec::with_capacity(uv.data.len() * 2);
        let mut uv_data: Vec<u8> = Vec::with_capacity(uv.data.len() * 2);
        uv_data = vec![128; uv.data.len() * 2];
        uv_data[0..uv.data.len()].copy_from_slice(&uv.data[..]);
        uv_data[uv.data.len()..].copy_from_slice(&uv.data[..]);
        // let uv_data: Vec<u8> = uv.data.iter().zip(uv.data.iter()).flat_map(|(a, b)| vec![a, b]).copied().collect();
        // for b in &uv.data[..] {
        //     uv_data.push(*b);
        //     uv_data.push(*b);
        // }

        let yuv = &[&ay.data[..], &uv_data[..]];

        let dst_sizes: &mut [usize] = &mut [0usize; 1];
        get_buffers_size(ay.width, ay.height, &BGRA, None, dst_sizes)?;
        trace!("yeet");

        let mut dst_rgba: Vec<_> = vec![0u8; dst_sizes[0]];
        let dst_buffers: &mut [&mut [u8]] = &mut [&mut dst_rgba[..]];

        convert_image(
            ay.width,
            ay.height,
            &NV12,
            None,
            yuv,
            &BGRA,
            None,
            dst_buffers,
        )?;

        Ok(dst_rgba)
    }

    #[cfg(feature = "image")]
    pub fn yuv_to_image(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, ErrorKind> {
        let rgba = self.yuv_to_bgra()?;
        let first = &self.mipmaps[0];
        let w = first.width;
        let h = first.height;
        let image = ImageBuffer::from_raw(w, h, rgba).unwrap();
        // Ok(DynamicImage::ImageRgba8(image))
        Ok(image)
    }
}
