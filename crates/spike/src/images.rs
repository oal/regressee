// SPDX-License-Identifier: MPL-2.0
//! PNG row decoding: never allocate the whole full-page image or upload it whole.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufReader, Read},
    path::Path,
    sync::atomic::AtomicBool,
};
pub const MAX_WIDTH: u32 = 4096;
pub const MAX_HEIGHT: u32 = 32768;
pub const MAX_PIXELS: u64 = 64_000_000;
pub const TILE_HEIGHT: u32 = 512;
#[derive(Serialize, Deserialize, Clone)]
pub struct Tiles {
    pub width: u32,
    pub height: u32,
    pub files: Vec<String>,
}
pub fn budget(width: u32, height: u32) -> Result<()> {
    ensure!(
        width > 0
            && height > 0
            && width <= MAX_WIDTH
            && height <= MAX_HEIGHT
            && u64::from(width) * u64::from(height) <= MAX_PIXELS,
        "Image exceeds spike safety limits (4096 wide, 32768 tall, 64M pixels); use a smaller capture"
    );
    Ok(())
}
pub fn tile(input: &Path, output: &Path) -> Result<Tiles> {
    tile_cancellable(input, output, &AtomicBool::new(false))
}
pub fn tile_cancellable(input: &Path, output: &Path, cancel: &AtomicBool) -> Result<Tiles> {
    crate::cancel::check(cancel, "image decoding")?;
    ensure!(
        fs::metadata(input)?.len() <= 256 * 1024 * 1024,
        "compressed image exceeds 256 MiB"
    );
    tile_reader(File::open(input)?, output, cancel)
}
fn tile_reader(input: impl Read, output: &Path, cancel: &AtomicBool) -> Result<Tiles> {
    let mut decoder = png::Decoder::new_with_limits(
        BufReader::new(crate::cancel::Cancellable {
            inner: input,
            cancel,
        }),
        png::Limits {
            bytes: 16 * 1024 * 1024,
        },
    );
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;
    let info = reader.info();
    let (width, height) = (info.width, info.height);
    budget(width, height)?;
    ensure!(
        !info.interlaced,
        "interlaced PNG is unsupported by bounded row decoder"
    );
    let (color, depth) = reader.output_color_type();
    ensure!(depth == png::BitDepth::Eight, "unsupported image depth");
    fs::create_dir_all(output)?;
    let generation = format!(
        "generation-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    );
    let stage = output.join(format!(".{generation}.stage"));
    let published = output.join(&generation);
    fs::create_dir(&stage)?;
    struct Stage(std::path::PathBuf);
    impl Drop for Stage {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _guard = Stage(stage.clone());
    let stride = width as usize * color.samples();
    let mut files = Vec::new();
    let mut y = 0;
    while y < height {
        let rows = TILE_HEIGHT.min(height - y);
        let mut pixels = Vec::with_capacity(rows as usize * stride);
        for _ in 0..rows {
            crate::cancel::check(cancel, "image decoding")?;
            pixels.extend_from_slice(reader.next_row()?.context("truncated image rows")?.data());
        }
        let name = format!("tile-{:05}.png", files.len());
        let dest = stage.join(&name);
        let mut encoder = png::Encoder::new(File::create(&dest)?, width, rows);
        encoder.set_color(color);
        encoder.set_depth(depth);
        encoder.write_header()?.write_image_data(&pixels)?;
        files.push(
            output
                .canonicalize()?
                .join(&generation)
                .join(name)
                .to_string_lossy()
                .into_owned(),
        );
        y += rows;
    }
    // Reading the expected rows does NOT consume final IDAT CRC / IEND.
    reader.finish().context("invalid complete PNG stream")?;
    crate::cancel::check(cancel, "image decoding")?;
    let tiles = Tiles {
        width,
        height,
        files,
    };
    fs::write(stage.join("tiles.json"), serde_json::to_vec_pretty(&tiles)?)?;
    // No viewer-visible file is modified before complete stream validation.
    fs::rename(&stage, &published)?;
    Ok(tiles)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn budgets() {
        assert!(budget(1440, 20000).is_ok());
        assert!(budget(4096, 32768).is_err());
        assert!(budget(1, 32769).is_err());
        assert!(budget(4097, 1).is_err());
        assert!(budget(0, 1).is_err());
    }
    #[test]
    fn tall_row_tiles() -> Result<()> {
        let scratch = tempfile::tempdir()?;
        let dir = scratch.path();
        let source = dir.join("generated.png");
        let mut e = png::Encoder::new(File::create(&source)?, 64, 20000);
        e.set_color(png::ColorType::Rgb);
        e.set_depth(png::BitDepth::Eight);
        e.write_header()?
            .write_image_data(&vec![120; 64 * 20000 * 3])?;
        let t = tile(&source, &dir.join("derived"))?;
        assert_eq!(t.files.len(), 40);
        assert_eq!(t.height, 20000);
        Ok(())
    }
    #[test]
    fn invalid_streams_preserve_displayed_generation() -> Result<()> {
        let scratch = tempfile::tempdir()?;
        let dir = scratch.path();
        let source = dir.join("valid.png");
        let mut e = png::Encoder::new(File::create(&source)?, 64, 1200);
        e.set_color(png::ColorType::Rgb);
        let mut seed = 1u32;
        let pixels: Vec<u8> = (0..64 * 1200 * 3)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                seed as u8
            })
            .collect();
        e.write_header()?.write_image_data(&pixels)?;
        let bytes = fs::read(&source)?;
        let output = dir.join("tiles");
        let displayed = tile(&source, &output)?;
        let original: Vec<_> = displayed
            .files
            .iter()
            .map(fs::read)
            .collect::<std::io::Result<_>>()?;
        struct CancellingReader<'a> {
            inner: std::io::Cursor<&'a [u8]>,
            cancel: &'a AtomicBool,
            output: &'a Path,
            saw_staged_tile: &'a AtomicBool,
        }
        impl Read for CancellingReader<'_> {
            fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
                let n = self.inner.read(b)?;
                if self.inner.position() as usize > self.inner.get_ref().len() * 2 / 3 {
                    let staged = fs::read_dir(self.output)?.flatten().any(|e| {
                        e.file_name().to_string_lossy().ends_with(".stage")
                            && e.path().join("tile-00000.png").exists()
                    });
                    self.saw_staged_tile
                        .store(staged, std::sync::atomic::Ordering::Relaxed);
                    self.cancel
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                }
                Ok(n)
            }
        }
        let cancel = AtomicBool::new(false);
        let saw_staged_tile = AtomicBool::new(false);
        let reader = CancellingReader {
            inner: std::io::Cursor::new(bytes.as_slice()),
            cancel: &cancel,
            output: &output,
            saw_staged_tile: &saw_staged_tile,
        };
        let error = match tile_reader(reader, &output, &cancel) {
            Ok(_) => panic!("mid-decode cancellation accepted"),
            Err(e) => format!("{e:#}"),
        };
        assert!(error.contains("cancelled"), "{error}");
        assert!(
            saw_staged_tile.load(std::sync::atomic::Ordering::Relaxed),
            "test did not reach staged tile work"
        );
        assert_eq!(fs::read_dir(&output)?.count(), 1, "cancelled stage leaked");
        for (path, before) in displayed.files.iter().zip(&original) {
            assert_eq!(&fs::read(path)?, before);
        }
        let mut final_crc = bytes.clone();
        // Find the last IDAT's CRC, not an arbitrary compressed byte.
        let mut p = 8;
        let mut crc = 0;
        while p < bytes.len() {
            let n = u32::from_be_bytes(bytes[p..p + 4].try_into()?) as usize;
            if &bytes[p + 4..p + 8] == b"IDAT" {
                crc = p + 8 + n;
            }
            p += n + 12;
        }
        assert!(crc > 0);
        final_crc[crc] ^= 1;
        let mut final_chunk = bytes.clone();
        *final_chunk.last_mut().unwrap() ^= 1;
        for (name, bad, reason) in [
            (
                "truncated",
                bytes[..bytes.len() * 3 / 4].to_vec(),
                "unexpected end of file",
            ),
            (
                "missing-IEND",
                bytes[..bytes.len() - 12].to_vec(),
                "unexpected end of file",
            ),
            ("final-IDAT-CRC", final_crc, "CRC"),
            ("IEND-CRC", final_chunk, "CRC"),
        ] {
            let source = dir.join(format!("{name}.png"));
            fs::write(&source, bad)?;
            let error = match tile(&source, &output) {
                Ok(_) => panic!("accepted {name}"),
                Err(e) => format!("{e:#}"),
            };
            assert!(error.contains(reason), "{name}: {error}");
            for (path, before) in displayed.files.iter().zip(&original) {
                assert_eq!(
                    &fs::read(path)?,
                    before,
                    "{name} changed displayed evidence"
                );
            }
            assert_eq!(fs::read_dir(&output)?.count(), 1, "failed stage leaked");
        }
        let error = match tile_cancellable(&source, &output, &AtomicBool::new(true)) {
            Ok(_) => panic!("accepted cancelled decode"),
            Err(e) => e.to_string(),
        };
        assert_eq!(error, "image decoding cancelled");
        Ok(())
    }
}
