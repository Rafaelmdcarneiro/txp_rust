use std::borrow::Cow;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "ddsfile")]
mod dds;
#[cfg(feature = "image")]
mod image;
mod r#impl;
#[cfg(feature = "pyo3")]
pub mod py_ffi;
mod read;
#[cfg(feature = "dcv-color-primitives")]
mod yuv;

#[derive(Debug, PartialEq, Clone)]
pub struct TextureAtlas<'a>(pub Vec<Texture<'a>>);

#[derive(Debug, PartialEq, Clone)]
pub struct Texture<'a> {
    pub subtextures: Vec<Subtexture<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Subtexture<'a> {
    pub mipmaps: Vec<Mipmap<'a>>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Mipmap<'a> {
    id: u32,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub data: Cow<'a, [u8]>,
}

#[non_exhaustive]
#[derive(Debug, Default, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "pyo3", pyclass)]
pub enum TextureFormat {
    A8 = 0,
    #[default]
    RGB8 = 1,
    RGBA8 = 2,
    RGB5 = 3,
    RGB5A1 = 4,
    RGBA4 = 5,
    DXT1 = 6,
    DXT1a = 7,
    DXT3 = 8,
    DXT5 = 9,
    ATI1 = 10,
    ATI2 = 11,
    L8 = 12,
    L8A8 = 13,
    /// Only available on MM+
    BC7 = 15,
    /// Only available on MM+
    BC6H = 127,
}
