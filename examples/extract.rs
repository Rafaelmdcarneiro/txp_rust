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

    ext: Option<String>,
}

use std::fs::File;
use std::io::Read;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let opt = Opt::from_args();
    let mut file = File::open(&opt.input)?;
    let mut data = vec![];
    file.read_to_end(&mut data)?;
    let (_, atlas) = TextureAtlas::parse(&data).unwrap();
    let path = opt
        .input
        .parent()
        .unwrap()
        .join(opt.input.file_stem().unwrap());
    std::fs::create_dir(&path);
    let ext = opt.ext.unwrap_or("png".into());
    for (i, tex) in atlas.0.into_iter().enumerate() {
        if ext == "dds" {
            let name = format!("tex{}.{}", i, ext);
            let path = path.join(name);
            let mut save = File::create(path)?;
            let dds = tex.to_dds()?;
            dds.write(&mut save)?;
        } else {
            if tex.subtextures.len() == 1 {
                let name = format!("tex{}.{}", i, ext);
                let path = path.join(name);
                let t = &tex.subtextures[0].mipmaps;
                image_extract(t[0].clone(), path);
            } else {
                for (j, side) in tex.subtextures.iter().enumerate() {
                    let name = format!("tex{}_sub{}.{}", i, j, ext);
                    let path = path.join(name);
                    image_extract(side.mipmaps[0].clone(), path);
                }
            }
        }
    }
    Ok(())
}

use std::path::Path;
fn image_extract<Q: AsRef<Path>>(subtex: Mipmap<'_>, path: Q) -> Option<()> {
    let image = subtex.to_dynamic_image()?;
    image.flipv().save(path);
    Some(())
}
