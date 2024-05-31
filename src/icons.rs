use std::io::Write;

use clio::{ClioPath, Output};
use image::{ImageBuffer, ImageEncoder};
use ironworks::{file::tex::{self, Format, Texture}, sqpack::{Install, Resource, SqPack}, Ironworks};
use crate::err::Err;
use crate::err::ToUnknownErr;

pub fn extract(output: &mut Output, id: u32, game_dir: &Option<ClioPath>) -> Result<(), Err> {
    let game_resource = if let Some(game_dir) = game_dir {
        Some(Install::at(game_dir.path()))
    } else {
        Install::search()
    }.ok_or(Err::GameNotFound)?;

    // There's currently an error in ironworks where search() always returns
    // Some(), even if no path was found. We do this check to ensure the path
    // actually points to the game.
    game_resource.version(0).map_err(|_| Err::GameNotFound)?;

    let ironworks = Ironworks::new().with_resource(SqPack::new(game_resource));
    let icon_path = get_icon_path(id);
    let file = ironworks.file::<tex::Texture>(&icon_path).map_err(|_| Err::IconNotFound(icon_path.to_owned()))?;
    write_as_png(output, &file, &icon_path)?;

    Ok(())
}

/// See https://github.com/xivapi/ffxiv-datamining/blob/master/docs/IconPaths.md
fn get_icon_path(id: u32) -> String {
    let id_str = id.to_string();

    let icon = if id_str.len() > 5 {
        id_str
    } else {
        format!("0{:0>5}", id_str)
    };

    format!("ui/icon/{}000/{}_hr1.tex", &icon[0..3], icon)
}

fn write_as_png<'a>(w: impl std::io::Write, file: &'a Texture, path: &str) -> Result<(), Err> {
    let mut output: Vec<u8> = vec![0; (4 * file.width() * file.height()).into()];
    let format = file.format();
    let texpresso_format = get_texpresso_format(file.format())
        .ok_or_else(|| Err::UnsupportedIconFormat(format as u32, path.to_owned()))?;

    texpresso_format.decompress(file.data(), file.width().into(), file.height().into(), &mut output);
    let encoder = image::codecs::png::PngEncoder::new(w);
    encoder.write_image(&output, file.width().into(), file.height().into(), image::ExtendedColorType::Rgba8).to_unknown_err()?;

    Ok(())
}

fn get_texpresso_format(format: Format) -> Option<texpresso::Format> {
    match format {
        Format::Dxt1 => Some(texpresso::Format::Bc1),
        Format::Dxt3 => Some(texpresso::Format::Bc3),
        Format::Dxt5 => Some(texpresso::Format::Bc5),
        _ => None
    }
}
