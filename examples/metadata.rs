use anyhow::*;
use structopt::StructOpt;
use txp::*;

use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

use std::fs::File;
use std::io::Read;

use std::io::Write;

use tabwriter::TabWriter;

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut file = File::open(opt.input)?;
    let mut data = vec![];
    file.read_to_end(&mut data)?;
    let (_, atlas) = TextureAtlas::parse(&data).unwrap();
    for (i, tex) in atlas.0.iter().enumerate() {
        println!("Texture #{}", i + 1);
        if tex.subtextures.len() == 1 {
            print_mips(&tex.subtextures[0].mipmaps, "\t")?;
        } else {
            for (j, subtex) in tex.subtextures.iter().enumerate() {
                println!("\tSubtexture #{} with {} mips", j + 1, subtex.mipmaps.len());
                print_mips(&subtex.mipmaps, "\t")?;
            }
        }
    }
    Ok(())
}

fn print_mips(mips: &[Mipmap<'_>], tab: &str) -> Result<()> {
    let mut tw = TabWriter::new(vec![]);
    for (i, mip) in mips.iter().enumerate() {
        // println!("\t{}", mip);
        write!(
            tw,
            "\t{}#{}\t{}x{}\t{:?}\n",
            tab,
            i + 1,
            mip.width,
            mip.height,
            mip.format
        )?;
    }
    println!("{}", String::from_utf8(tw.into_inner()?)?);
    Ok(())
}
