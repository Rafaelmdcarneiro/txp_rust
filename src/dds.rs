use ddsfile;
use ddsfile::AlphaMode;
use ddsfile::D3D10ResourceDimension;
use ddsfile::Dds;
use ddsfile::NewD3dParams;
use ddsfile::{D3DFormat, DxgiFormat};
use tracing::{debug, instrument};

use std::convert::TryInto;

use super::*;

impl Texture<'_> {
    fn caps2() -> ddsfile::Caps2 {
        use ddsfile::Caps2;
        let mut caps = Caps2::all();
        caps.remove(Caps2::VOLUME);
        caps
    }
    #[tracing::instrument(skip(self))]
    fn d3d(&self) -> Result<Dds, ddsfile::Error> {
        let def = Default::default();
        let first = self
            .subtextures
            .get(0)
            .and_then(|x| x.mipmaps.get(0))
            .unwrap_or(&def);
        let format = first
            .format
            .to_d3d_format()
            .ok_or(ddsfile::Error::UnsupportedFormat)?;
        let mipmap_levels = self
            .subtextures
            .get(0)
            .and_then(|x| x.mipmaps.len().try_into().ok());
        let caps2 = Some(Self::caps2()).filter(|_| self.subtextures.len() == 6);
        let params = NewD3dParams {
            height: first.height,
            width: first.width,
            depth: None,
            format,
            mipmap_levels,
            caps2,
        };
        debug!(first.height, first.width, mipmap_levels, ?caps2);
        Dds::new_d3d(params)
    }
    #[tracing::instrument(skip(self))]
    fn dxgi(&self) -> Result<Dds, ddsfile::Error> {
        use TextureFormat::*;
        let def = Default::default();
        let first = self
            .subtextures
            .get(0)
            .and_then(|x| x.mipmaps.get(0))
            .unwrap_or(&def);
        let format = first.format.to_dxgi_format();
        let alpha_mode = match first.format {
            DXT1 | DXT1a => AlphaMode::PreMultiplied,
            _ => AlphaMode::Straight,
        };
        let mipmap_levels = self.subtextures.get(0).map(|x| x.mipmaps.len() as u32);
        let array_layers = self.subtextures.len().try_into().ok().filter(|&x| x > 1);
        let caps2 = Some(Self::caps2()).filter(|_| self.subtextures.len() == 6);
        let is_cubemap = self.subtextures.len() == 6;
        let params = ddsfile::NewDxgiParams {
            height: first.height,
            width: first.width,
            depth: None,
            format,
            mipmap_levels,
            array_layers,
            caps2,
            is_cubemap,
            resource_dimension: D3D10ResourceDimension::Texture2D,
            alpha_mode,
        };
        debug!(
            first.height,
            first.width,
            mipmap_levels,
            array_layers,
            ?caps2,
            is_cubemap,
            ?alpha_mode,
        );
        Dds::new_dxgi(params)
    }
    #[tracing::instrument(skip(self))]
    pub fn to_dds(&self) -> Result<Dds, ddsfile::Error> {
        let dds = self.d3d().or_else(|_| self.dxgi());
        dds.map(|mut x| {
            x.data = self
                .subtextures
                .iter()
                .flat_map(|x| x.mipmaps.iter().flat_map(|x| &x.data[..]))
                .cloned()
                .collect();
            x
        })
    }
}

impl TextureFormat {
    #[tracing::instrument(level = "trace", ret)]
    pub fn to_d3d_format(&self) -> Option<D3DFormat> {
        use TextureFormat::*;
        match self {
            A8 => Some(D3DFormat::A8),
            RGB8 => Some(D3DFormat::R8G8B8),
            RGBA8 => Some(D3DFormat::A8R8G8B8),
            RGB5 => Some(D3DFormat::R5G6B5),
            RGB5A1 => Some(D3DFormat::A1R5G5B5),
            RGBA4 => Some(D3DFormat::A4R4G4B4),
            DXT1 => Some(D3DFormat::DXT1),
            DXT1a => Some(D3DFormat::DXT1),
            DXT3 => Some(D3DFormat::DXT3),
            DXT5 => Some(D3DFormat::DXT5),
            L8 => Some(D3DFormat::L8),
            L8A8 => Some(D3DFormat::A8L8),
            _ => None,
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn to_dxgi_format(&self) -> DxgiFormat {
        use TextureFormat::*;
        match self {
            A8 => DxgiFormat::A8_UNorm,
            RGB8 | RGBA8 => DxgiFormat::R8G8B8A8_UNorm,
            RGB5 => DxgiFormat::B5G6R5_UNorm,
            RGB5A1 => DxgiFormat::B5G5R5A1_UNorm,
            RGBA4 => DxgiFormat::B4G4R4A4_UNorm,
            DXT1 | DXT1a => DxgiFormat::BC1_UNorm,
            DXT3 => DxgiFormat::BC2_UNorm,
            DXT5 => DxgiFormat::BC3_UNorm,
            ATI1 => DxgiFormat::BC4_UNorm,
            ATI2 => DxgiFormat::BC5_UNorm,
            L8 => DxgiFormat::A8_UNorm,
            L8A8 => DxgiFormat::A8P8,
            BC7 => DxgiFormat::BC7_UNorm,
            BC6H => DxgiFormat::BC6H_Typeless,
        }
    }
}
