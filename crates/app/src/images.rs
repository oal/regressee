// SPDX-License-Identifier: MPL-2.0
//! Bounded PNG rows, 512-row textures, nearest-neighbour thumbnail, initial diff.
use anyhow::{Context, Result, ensure};
use regressee_core::{domain::*, evidence::*};
use regressee_runtime::distribution::check_cancel;
use regressee_storage::Store;
use std::{
    fs::{self, File},
    io::{BufReader, Read, Seek, SeekFrom, Write},
    path::Path,
    sync::atomic::AtomicBool,
};
pub const TILE_HEIGHT: u32 = 512;
pub fn budget(w: u32, h: u32) -> Result<()> {
    ensure!(
        w > 0 && w <= 4096 && h > 0 && h <= 32768 && u64::from(w) * u64::from(h) <= 64_000_000,
        "image exceeds limits (4096 wide, 32768 tall, 64M pixels)"
    );
    Ok(())
}
struct Rows<R: Read + Seek = File> {
    reader: png::Reader<BufReader<R>>,
    w: u32,
    h: u32,
    color: png::ColorType,
}
impl Rows {
    fn open(path: &Path) -> Result<Self> {
        let meta = fs::symlink_metadata(path)?;
        ensure!(
            meta.is_file()
                && regressee_core::platform::ordinary(&meta)
                && meta.len() <= 128 * 1024 * 1024,
            "unsafe image file"
        );
        Self::read(File::open(path)?, meta.len())
    }
}
impl<R: Read + Seek> Rows<R> {
    fn read(mut f: R, length: u64) -> Result<Self> {
        ensure!(
            (20..=128 * 1024 * 1024).contains(&length),
            "truncated or oversized PNG"
        );
        let mut signature = [0; 8];
        f.read_exact(&mut signature)?;
        ensure!(&signature == b"\x89PNG\r\n\x1a\n", "not a PNG screenshot");
        // A trailing PNG also ends with IEND: checking only the last twelve
        // bytes misses concatenated files. Require the FIRST IEND at exact EOF.
        let mut chunks = 0;
        loop {
            chunks += 1;
            ensure!(chunks <= 10000, "PNG chunk count exceeds limit");
            let mut header = [0; 8];
            f.read_exact(&mut header)?;
            let n = u32::from_be_bytes(header[..4].try_into().unwrap()) as u64;
            ensure!(
                f.stream_position()?
                    .checked_add(n + 4)
                    .is_some_and(|end| end <= length),
                "PNG chunk exceeds file bounds"
            );
            f.seek(SeekFrom::Current((n + 4) as i64))?;
            if &header[4..] == b"IEND" {
                ensure!(
                    n == 0 && f.stream_position()? == length,
                    "PNG trailing data or malformed terminator"
                );
                break;
            }
        }
        f.rewind()?;
        let mut decoder = png::Decoder::new_with_limits(
            BufReader::new(f),
            png::Limits {
                bytes: 16 * 1024 * 1024,
            },
        );
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let reader = decoder.read_info()?;
        let i = reader.info();
        budget(i.width, i.height)?;
        ensure!(
            !i.interlaced && i.animation_control.is_none(),
            "interlaced/animated PNG unsupported"
        );
        let (color, depth) = reader.output_color_type();
        ensure!(depth == png::BitDepth::Eight, "unsupported PNG depth");
        Ok(Self {
            w: i.width,
            h: i.height,
            color,
            reader,
        })
    }
    fn row(&mut self, cancel: &AtomicBool) -> Result<Vec<u8>> {
        check_cancel(cancel)?;
        let data = self.reader.next_row()?.context("truncated PNG row")?;
        let data = data.data();
        let mut rgba = Vec::with_capacity(self.w as usize * 4);
        for p in data.chunks_exact(self.color.samples()) {
            match self.color {
                png::ColorType::Rgba => rgba.extend_from_slice(p),
                png::ColorType::Rgb => rgba.extend_from_slice(&[p[0], p[1], p[2], 255]),
                png::ColorType::Grayscale => rgba.extend_from_slice(&[p[0], p[0], p[0], 255]),
                png::ColorType::GrayscaleAlpha => rgba.extend_from_slice(&[p[0], p[0], p[0], p[1]]),
                _ => anyhow::bail!("unsupported PNG color"),
            }
        }
        ensure!(rgba.len() == self.w as usize * 4, "PNG row size mismatch");
        Ok(rgba)
    }
    fn finish(&mut self) -> Result<()> {
        self.reader.finish().context("invalid complete PNG stream")
    }
}
#[cfg(test)]
fn encode(path: &Path, w: u32, h: u32, pixels: &[u8], purpose: &str) -> Result<()> {
    let mut e = png::Encoder::new(
        File::options().create_new(true).write(true).open(path)?,
        w,
        h,
    );
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.add_text_chunk("Regressee derivative".into(), purpose.into())?;
    e.write_header()?.write_image_data(pixels)?;
    Ok(())
}
/// Decode a bounded, verified tile derivative without native graphics types.
pub fn decode_tile(store: &Store, id: &ArtifactId) -> Result<(u32, u32, Vec<u8>)> {
    let (artifact, bytes) = store.read_verified_artifact(id, 16 * 1024 * 1024)?;
    ensure!(
        artifact.kind == ArtifactKind::Tile,
        "viewer only accepts tile derivatives"
    );
    let mut rows = Rows::read(std::io::Cursor::new(bytes), artifact.bytes)?;
    ensure!(
        rows.h <= TILE_HEIGHT,
        "viewer cannot upload original tall images"
    );
    let mut pixels = Vec::with_capacity(rows.w as usize * rows.h as usize * 4);
    let cancel = AtomicBool::new(false);
    for _ in 0..rows.h {
        let row = rows.row(&cancel)?;
        pixels.extend_from_slice(&row);
    }
    rows.finish()?;
    Ok((rows.w, rows.h, pixels))
}
pub fn ingest(
    store: &mut Store,
    path: &Path,
    kind: ArtifactKind,
    cancel: &AtomicBool,
) -> Result<ImageEvidence> {
    regressee_storage::images::ingest(store, path, kind, cancel)
}
pub fn compare(
    store: &mut Store,
    before: &ImageEvidence,
    after: &ImageEvidence,
    key: TargetKey,
    cancel: &AtomicBool,
) -> Result<(AnalysisResult, ImageEvidence)> {
    let mut a = Rows::open(&store.artifact_path(&before.original)?)?;
    let mut b = Rows::open(&store.artifact_path(&after.original)?)?;
    ensure!(
        a.w == before.width
            && a.h == before.height
            && b.w == after.width
            && b.h == after.height
            && a.w == b.w,
        "incompatible image width/profile"
    );
    let w = a.w;
    let h = a.h.max(b.h);
    budget(w, h)?;
    let stage = tempfile::tempdir_in(&store.paths().cache)?;
    let p = stage.path().join("heatmap.png");
    let mut encoder = png::Encoder::new(File::create(&p)?, w, h);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.add_text_chunk("Regressee analysis".into(), "rgba-max-channel-v1".into())?;
    let mut writer = encoder.write_header()?;
    let mut changed = 0u64;
    {
        let mut stream = writer.stream_writer_with_size(64 * 1024)?;
        for y in 0..h {
            check_cancel(cancel)?;
            let ar = if y < a.h { Some(a.row(cancel)?) } else { None };
            let br = if y < b.h { Some(b.row(cancel)?) } else { None };
            let mut heat = Vec::with_capacity(w as usize * 4);
            for x in 0..w as usize {
                let offset = x * 4;
                let difference = match (&ar, &br) {
                    (Some(a), Some(b)) => a[offset..offset + 4]
                        .iter()
                        .zip(&b[offset..offset + 4])
                        .any(|(a, b)| a.abs_diff(*b) > 16),
                    _ => true,
                };
                if difference {
                    changed += 1;
                    heat.extend_from_slice(&[220, 50, 70, 255]);
                } else {
                    let v = br.as_ref().map(|b| b[offset]).unwrap_or(255) / 4 + 180;
                    heat.extend_from_slice(&[v, v, v, 255]);
                }
            }
            stream.write_all(&heat)?;
        }
        stream.finish()?;
    }
    writer.finish()?;
    a.finish()?;
    b.finish()?;
    check_cancel(cancel)?;
    let result = AnalysisResult {
        key,
        classification: if changed > 0 {
            Classification::Changed
        } else {
            Classification::NoSignificantChange
        },
        eligible_pixels: u64::from(w) * u64::from(h),
        changed_pixels: changed,
        dimensions_changed: a.h != b.h,
    };
    result.validate()?;
    Ok((result, ingest(store, &p, ArtifactKind::Difference, cancel)?))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn viewer_tile_exact_bytes_kind_png_and_dimensions_remain_authoritative() -> Result<()> {
        let root = tempfile::tempdir()?;
        let paths = regressee_storage::AppPaths::isolated(root.path());
        let mut store = Store::open(paths.clone())?;
        let path = root.path().join("tile.png");
        encode(&path, 8, 2, &[255; 8 * 2 * 4], "synthetic viewer tile")?;
        let bytes = fs::read(&path)?;
        let tile = store.put_artifact(bytes.as_slice(), ArtifactKind::Tile)?;
        let (_, _, pixels) = decode_tile(&store, &tile.id)?;
        assert_eq!(pixels.len(), 8 * 2 * 4);
        let joined = [bytes.as_slice(), bytes.as_slice()].concat();
        let trailing = store.put_artifact(joined.as_slice(), ArtifactKind::Tile)?;
        assert!(decode_tile(&store, &trailing.id).is_err());
        let large = root.path().join("large.png");
        encode(
            &large,
            8,
            513,
            &vec![255; 8 * 513 * 4],
            "oversized viewer tile",
        )?;
        let large = store.put_artifact(fs::File::open(large)?, ArtifactKind::Tile)?;
        assert!(decode_tile(&store, &large.id).is_err());
        let invalid = store.put_artifact(&b"not a PNG"[..], ArtifactKind::Tile)?;
        assert!(decode_tile(&store, &invalid.id).is_err());
        let wrong = store.put_artifact(&b"not a tile"[..], ArtifactKind::Screenshot)?;
        assert!(decode_tile(&store, &wrong.id).is_err());
        let mut damaged = bytes;
        damaged[20] ^= 1;
        fs::write(paths.artifacts().join(tile.id.as_str()), damaged)?;
        assert!(decode_tile(&store, &tile.id).is_err());
        Ok(())
    }
    #[test]
    #[ignore = "read-only interop confirmation against retained actual product browser PNGs"]
    fn retained_product_originals_pass_current_row_gate() -> Result<()> {
        let paths = std::env::var_os("REGRESSEE_PHASE2_PNGS")
            .context("actual retained PNG paths required")?;
        let root = tempfile::tempdir()?;
        let mut store = Store::open(regressee_storage::AppPaths::isolated(root.path()))?;
        let cancel = AtomicBool::new(false);
        let mut count = 0;
        for path in std::env::split_paths(&paths) {
            let image = ingest(&mut store, &path, ArtifactKind::Screenshot, &cancel)?;
            assert_eq!(image.tiles.len(), (image.height as usize).div_ceil(512));
            count += 1;
        }
        ensure!(
            count >= 3,
            "include unchanged, changed and tall actual originals"
        );
        println!(
            "Validated {count} retained actual product screenshots with the current complete PNG/chunk/CRC gate"
        );
        Ok(())
    }
    #[test]
    fn tall_diff_missing_band_and_corrupt_stream() {
        let root = tempfile::tempdir().unwrap();
        let mut s = Store::open(regressee_storage::AppPaths::isolated(root.path())).unwrap();
        let c = AtomicBool::new(false);
        let a = root.path().join("a.png");
        let b = root.path().join("b.png");
        encode(&a, 8, 20000, &vec![255; 8 * 20000 * 4], "test a").unwrap();
        encode(&b, 8, 19999, &vec![255; 8 * 19999 * 4], "test b").unwrap();
        let a = ingest(&mut s, &a, ArtifactKind::Screenshot, &c).unwrap();
        let b = ingest(&mut s, &b, ArtifactKind::Screenshot, &c).unwrap();
        assert_eq!(a.tiles.len(), 40);
        let key = TargetKey {
            page: PageId::new(),
            profile: ProfileId::new(),
        };
        let (r, h) = compare(&mut s, &a, &b, key.clone(), &c).unwrap();
        assert_eq!(r.changed_pixels, 8);
        assert!(r.dimensions_changed);
        assert_eq!(h.tiles.len(), 40);
        let (r, _) = compare(&mut s, &a, &a, key, &c).unwrap();
        assert_eq!(r.classification, Classification::NoSignificantChange);
        let bytes = fs::read(s.artifact_path(&a.original).unwrap()).unwrap();
        let p = root.path().join("bad.png");
        fs::write(&p, &bytes[..bytes.len() - 12]).unwrap();
        let before = s.summary().unwrap().artifacts;
        assert!(ingest(&mut s, &p, ArtifactKind::Screenshot, &c).is_err());
        assert_eq!(s.summary().unwrap().artifacts, before);
        let mut bad = bytes;
        let n = bad.len();
        bad[n - 13] ^= 1;
        fs::write(&p, bad).unwrap();
        assert!(ingest(&mut s, &p, ArtifactKind::Screenshot, &c).is_err());
        assert_eq!(s.summary().unwrap().artifacts, before);
        let valid = fs::read(s.artifact_path(&a.original).unwrap()).unwrap();
        fs::write(&p, [valid.as_slice(), valid.as_slice()].concat()).unwrap();
        let error = ingest(&mut s, &p, ArtifactKind::Screenshot, &c).unwrap_err();
        assert!(format!("{error:#}").contains("PNG trailing bytes after IEND"));
        assert_eq!(s.summary().unwrap().artifacts, before);
    }
}
