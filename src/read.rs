use std::convert::TryInto;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::combinator::map_opt;
use nom::error::ParseError;
use nom::multi::count;
use nom::multi::length_data;
use nom::number::complete::u32;
use nom::IResult;
use nom::Parser;
use tracing::{debug, instrument, trace};

use super::*;

fn parse_magic(id: u8) -> impl Fn(&[u8]) -> IResult<&[u8], nom::number::Endianness> {
    use nom::number::Endianness::*;
    move |i: &[u8]| {
        let (i, res) = alt((tag(&[0x54, 0x58, 0x50, id]), tag(&[id, 0x50, 0x58, 0x54])))(i)?;
        Ok((i, if res[3] == id { Little } else { Big }))
    }
}

impl<'a> TextureAtlas<'a> {
    #[tracing::instrument(name = "atlas", skip(i0))]
    pub fn parse(i0: &'a [u8]) -> IResult<&'a [u8], TextureAtlas<'a>> {
        let (i, endian) = parse_magic(3)(i0)?;
        let (i, map_count) = u32(endian)(i)?;
        let (i, unk) = u32(endian)(i)?;
        debug!(?endian, map_count, unk);
        let parse = alt((Texture::parse, Texture::parse_array));
        let (_, maps) = offset_table(i0, parse, map_count.try_into().unwrap(), endian).parse(i)?;
        Ok((i, Self(maps)))
    }
}

impl<'a> Texture<'a> {
    #[tracing::instrument(name = "texture", skip(i0))]
    pub fn parse(i0: &'a [u8]) -> IResult<&'a [u8], Texture<'a>> {
        let (i, endian) = parse_magic(4)(i0)?;
        let (i, mip_count) = u32(endian)(i)?;
        let (i, unk) = u32(endian)(i)?;
        debug!(?endian, mip_count, unk);
        let (_, mipmaps) =
            offset_table(i0, Mipmap::parse, mip_count.try_into().unwrap(), endian).parse(i)?;
        Ok((
            i,
            Self {
                subtextures: vec![Subtexture { mipmaps }],
            },
        ))
    }
    #[tracing::instrument(name = "array", skip(i0))]
    pub fn parse_array(i0: &'a [u8]) -> IResult<&'a [u8], Texture<'a>> {
        use nom::multi::count;
        let (i, endian) = parse_magic(5)(i0)?;
        let (i, total_mip_count) = u32(endian)(i)?;
        let (i, mipdata) = u32(endian)(i)?;
        let depth = (mipdata & 0xFF00) >> 8;
        let mip_count = total_mip_count / depth;
        let (_, subtextures) = count(
            offset_table(i0, Mipmap::parse, mip_count.try_into().unwrap(), endian)
                .map(|mipmaps| Subtexture { mipmaps }),
            depth as usize,
        )
        .parse(i)?;
        //let sides = sides.into_iter().map(|mipmaps| Side { mipmaps }).collect();
        Ok((i, Self { subtextures }))
    }
}

use nom::{InputIter, InputTake};
pub fn at_offset<I, O, E, F>(offset: usize, mut f: F) -> impl Parser<I, O, E>
where
    I: InputIter + InputTake + Clone,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    use nom::bytes::complete::*;
    move |i: I| {
        let (i0, _) = take(offset)(i.clone())?;
        let (_, v) = f.parse(i0)?;
        Ok((i, v))
    }
}

fn offset_table<'a, F, O, E>(
    i0: &'a [u8],
    mut f: F,
    cnt: usize,
    endian: nom::number::Endianness,
) -> impl Parser<&'a [u8], Vec<O>, E>
where
    F: Parser<&'a [u8], O, E>,
    E: ParseError<&'a [u8]>,
{
    move |i: &'a [u8]| {
        // let (i1, offset) = f1(i)?;
        let f1 = u32(endian)(i)?;
        let (i1, offsets) = count(u32(endian).map(|x| x as usize), cnt)(i)?;
        let mut res = vec![];
        let mut f0 = |x: &'a [u8]| f.parse(x);
        for offset in offsets {
            let (_, val) = at_offset(offset, &mut f0).parse(i0)?;
            res.push(val);
        }
        Ok((i1, res))
    }
}

impl<'a> Mipmap<'a> {
    #[tracing::instrument(name = "mip", skip(i))]
    pub fn parse(i: &'a [u8]) -> IResult<&'a [u8], Mipmap<'a>> {
        let (i, endian) = parse_magic(2)(i)?;
        let (i, width) = u32(endian)(i)?;
        let (i, height) = u32(endian)(i)?;
        let (i, format) = map_opt(u32(endian), TextureFormat::from_id)(i)?;
        let (i, id) = u32(endian)(i)?;
        let (i, data) = length_data(u32(endian))(i)?;
        let data = data.into();
        trace!(width, height, ?format, id);
        Ok((
            i,
            Self {
                width,
                height,
                format,
                id,
                data,
            },
        ))
    }
}

impl TextureFormat {
    #[tracing::instrument(error)]
    pub(crate) fn from_id(id: u32) -> Option<Self> {
        use super::TextureFormat::*;
        match id {
            0 => Some(Self::A8),
            1 => Some(Self::RGB8),
            2 => Some(Self::RGBA8),
            3 => Some(Self::RGB5),
            4 => Some(Self::RGB5A1),
            5 => Some(Self::RGBA4),
            6 => Some(Self::DXT1),
            7 => Some(Self::DXT1a),
            8 => Some(Self::DXT3),
            9 => Some(Self::DXT5),
            10 => Some(Self::ATI1),
            11 => Some(Self::ATI2),
            12 => Some(Self::L8),
            13 => Some(Self::L8A8),
            15 => Some(Self::BC7),
            127 => Some(Self::BC6H),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &[u8] = include_bytes!("../assets/mikitm001_tex.txp");
    const TEX_OFF: usize = 84;
    const MIP_OFF: usize = 100;

    #[test]
    fn read_subtexture() {
        let input = &INPUT[MIP_OFF..];
        let (_, mip) = Mipmap::parse(input).unwrap();
        assert_eq!(mip.id, 0);
        assert_eq!(mip.width, 256);
        assert_eq!(mip.height, 8);
        assert_eq!(mip.format, TextureFormat::RGB8);
    }

    #[test]
    fn read_texture() {
        let input = &INPUT[TEX_OFF..];
        let (_, tex) = Texture::parse(input).unwrap();
        println!("{:?}", tex);
        assert_eq!(tex.subtextures[0].mipmaps.len(), 1);
    }

    #[test]
    fn read_atlas() {
        let (_, atlas) = TextureAtlas::parse(INPUT).unwrap();
    }
}
