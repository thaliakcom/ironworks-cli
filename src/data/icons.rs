use image::ImageEncoder;
use ironworks::{file::tex::{self, Format, Texture}, sqpack::SqPack, Ironworks};
use crate::err::{Err, ToUnknownErr};
use super::Args;

/// Extracts an icon from the game files by ID and prints
/// it to [`stdout`] as a PNG.
pub fn extract(id: u32, args: &mut Args<impl std::io::Write>) -> Result<(), Err> {
    let game_resource = super::get_game_resource(&args.game_path.as_deref())?;
    let ironworks = Ironworks::new().with_resource(SqPack::new(game_resource));
    let icon_path = get_icon_path(id);
    let file = ironworks.file::<tex::Texture>(&icon_path).map_err(|_| Err::IconNotFound(icon_path.to_owned()))?;
    let mut out = args.out.as_mut().ok_or(Err::IconMissingOut)?;
    write_png(&file, &icon_path, &mut out)?;

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

fn write_png(file: &Texture, path: &str, out: &mut impl std::io::Write) -> Result<(), Err> {
    let width = file.width() as u32;
    let height = file.height() as u32;
    let mut output: Vec<u8> = vec![0; (4 * width * height) as usize];

    file.decompress(path, &mut output)?;

    let encoder = image::codecs::png::PngEncoder::new(out);
    encoder.write_image(&output, width, height, image::ExtendedColorType::Rgba8).to_unknown_err()?;

    Ok(())
}

trait TextureDecompressor {
    /// The texture image data is a byte array of an arbitrary shape
    /// depending on its format.
    ///
    /// This function decompresses this byte array, converting it
    /// into a byte array (usually RGBA8, 4 channels with 8 bits per channel)
    /// that [`image::codecs::png::PngEncoder`] can understand.
    fn decompress(&self, path: &str, output: &mut Vec<u8>) -> Result<(), Err>;
}


impl TextureDecompressor for Texture {
    fn decompress(&self, path: &str, output: &mut Vec<u8>) -> Result<(), Err> {
        let width: usize = self.width() as usize;
        let height: usize = self.height() as usize;
        let data = self.data();
        let format = self.format();
    
        match format {
            // Dxt1–3 (aka Bc1–3) are known image compression formats.
            Format::Dxt1 => texpresso::Format::Bc1.decompress(data, width, height, output),
            Format::Dxt3 => texpresso::Format::Bc3.decompress(data, width, height, output),
            Format::Dxt5 => texpresso::Format::Bc5.decompress(data, width, height, output),
            Format::Rgb5a1 => {
                // Image data is in R5G5B5A1 format (5 bits per RGB channel, 1 alpha bit,
                // for a total of 16 bits per pixel).
                // We iterate over each set of 2 array elements, combine those 2 array
                // elements to get a u16 (one pixel), then extract the bits corresponding
                // to each color channel accordingly and expand them to u32s, then narrow
                // them again to one u8 per color channel for the output array.

                for (i, chunk) in data.chunks(2).enumerate() {
                    let value = u16::from_le_bytes(chunk.try_into().to_unknown_err()?);

                    let a = (value & 0x8000) as u32;
                    let r = (value & 0x7C00) as u32;
                    let g = (value & 0x03E0) as u32;
                    let b = (value & 0x001F) as u32;

                    let rgb = (r << 9) | (g << 6) | (b << 3);
                    let argb = (a * 0x1FE00) | rgb | ((rgb >> 5) & 0x070707);

                    output[i * 4]     = (argb >> 16) as u8;
                    output[i * 4 + 1] = (argb >>  8) as u8;
                    output[i * 4 + 2] =  argb        as u8;
                    output[i * 4 + 3] = (argb >> 24) as u8;
                }
            },
            Format::Rgba4 => {
                // Image data is in R4G4B4A4 format (i.e. 4 bits per RGBA channel
                // for a total of 16 bits per pixel).
                // We iterate over each set of 2 array elements, combine those 2 array
                // elements to get a u16 (one pixel), then extract the bits corresponding
                // to each color channel accordingly and expand them into u8s.

                for (i, chunk) in data.chunks(2).enumerate() {
                    let value = u16::from_le_bytes(chunk.try_into().to_unknown_err()?);

                    output[i * 4]     = (((value >>  8) & 0x0F) << 4) as u8;
                    output[i * 4 + 1] = (((value >>  4) & 0x0F) << 4) as u8;
                    output[i * 4 + 2] =  ((value & 0x0F)        << 4) as u8;
                    output[i * 4 + 3] = (((value >> 12) & 0x0F) << 4) as u8;
                }
            },
            // Rgba8 is already in the right format, so we don't need to do anything.
            Format::Rgba8 => output.copy_from_slice(data),
            Format::Argb8 => {
                // Input has the right size, but it's in the wrong order, so
                // we move the bits around.

                let mut i = 0;
                let len = data.len();

                while i < len {
                    output[i]     = data[i + 2];
                    output[i + 1] = data[i + 1];
                    output[i + 2] = data[i    ];
                    output[i + 3] = data[i + 3];

                    i += 4;
                }
            },
            _ => Err(Err::UnsupportedIconFormat(format as u32, path.to_owned()))?
        };

        Ok(())
    }
}
