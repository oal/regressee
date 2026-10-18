// SPDX-License-Identifier: MPL-2.0
//! Bounded complete PNG validation and rebuildable derivatives, outside UI thread.
use super::*;
use regressee_core::evidence::ImageEvidence;
use std::{
    io::{BufReader, Seek},
    sync::atomic::{AtomicBool, Ordering},
};
pub const TILE_HEIGHT: u32 = 512;
pub fn budget(w: u32, h: u32) -> Result<()> {
    ensure!(
        w > 0 && w <= 4096 && h > 0 && h <= 32768 && u64::from(w) * u64::from(h) <= 64_000_000,
        "image exceeds limits (4096 wide, 32768 tall, 64M pixels)"
    );
    Ok(())
}
pub fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "Operation cancelled; retained evidence is unchanged"
    );
    Ok(())
}
pub struct Rows {
    reader: png::Reader<BufReader<fs::File>>,
    pub w: u32,
    pub h: u32,
    color: png::ColorType,
}
impl Rows {
    pub fn open(path: &Path, cancel: &AtomicBool) -> Result<Self> {
        check_cancel(cancel)?;
        ensure!(
            fs::symlink_metadata(path)?.is_file(),
            "PNG is missing or not a regular file"
        );
        let mut f = fs::File::open(path)?;
        let len = f.metadata()?.len();
        ensure!((20..=MAX_ARTIFACT_BYTES).contains(&len), "unsafe PNG size");
        // png::Reader::finish can discard excess inflated rows and compressed
        // trailing data. Share the audited analysis gate BEFORE any derivative
        // or CAS publication, using the same open file and bounded inflate window.
        regressee_core::analysis_engine::validate_png(&mut f, cancel)
            .context("invalid complete PNG stream")?;
        f.rewind()?;
        let mut decoder = png::Decoder::new_with_limits(
            BufReader::new(f),
            png::Limits {
                bytes: 16 * 1024 * 1024,
            },
        );
        decoder.ignore_checksums(false);
        decoder.set_ignore_text_chunk(true);
        decoder.set_ignore_iccp_chunk(true);
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
    pub fn row(&mut self, cancel: &AtomicBool) -> Result<Vec<u8>> {
        check_cancel(cancel)?;
        let data = self.reader.next_row()?.context("truncated PNG row")?;
        let mut rgba = Vec::with_capacity(self.w as usize * 4);
        for p in data.data().chunks_exact(self.color.samples()) {
            match self.color {
                png::ColorType::Rgba => rgba.extend_from_slice(p),
                png::ColorType::Rgb => rgba.extend_from_slice(&[p[0], p[1], p[2], 255]),
                png::ColorType::Grayscale => rgba.extend_from_slice(&[p[0], p[0], p[0], 255]),
                png::ColorType::GrayscaleAlpha => rgba.extend_from_slice(&[p[0], p[0], p[0], p[1]]),
                _ => bail!("unsupported PNG color"),
            }
        }
        ensure!(rgba.len() == self.w as usize * 4, "PNG row size mismatch");
        Ok(rgba)
    }
    pub fn finish(&mut self) -> Result<()> {
        ensure!(self.reader.next_row()?.is_none(), "PNG excess rows");
        self.reader.finish().context("invalid complete PNG stream")
    }
}
pub fn validate_png(path: &Path, cancel: &AtomicBool) -> Result<(u32, u32)> {
    let mut r = Rows::open(path, cancel)?;
    for _ in 0..r.h {
        r.row(cancel)?;
    }
    r.finish()?;
    Ok((r.w, r.h))
}
fn encode(path: &Path, w: u32, h: u32, pixels: &[u8], purpose: &str) -> Result<()> {
    let mut e = png::Encoder::new(create_private(path)?, w, h);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.add_text_chunk("Regressee derivative".into(), purpose.into())?;
    e.write_header()?.write_image_data(pixels)?;
    Ok(())
}
pub fn ingest(
    store: &mut Store,
    path: &Path,
    kind: ArtifactKind,
    cancel: &AtomicBool,
) -> Result<ImageEvidence> {
    ensure!(
        kind != ArtifactKind::DomSnapshot,
        "DOM snapshots use canonical text ingestion, not PNG processing"
    );
    let mut rows = Rows::open(path, cancel)?;
    let (w, h) = (rows.w, rows.h);
    crate::portability::require_space(&store.paths.cache, u64::from(w) * u64::from(h) * 4)?;
    let stage = tempfile::tempdir_in(&store.paths.cache)?;
    let ratio = (240.0 / w as f64).min(160.0 / h as f64).min(1.0);
    let tw = ((w as f64 * ratio) as u32).max(1);
    let th = ((h as f64 * ratio) as u32).max(1);
    let mut thumb = vec![0; tw as usize * th as usize * 4];
    let mut ty = 0;
    let mut tiles = Vec::new();
    let mut y = 0;
    while y < h {
        let count = TILE_HEIGHT.min(h - y);
        let mut pixels = Vec::with_capacity(w as usize * count as usize * 4);
        for dy in 0..count {
            let row = rows.row(cancel)?;
            while ty < th && ty * h / th == y + dy {
                for tx in 0..tw {
                    let sx = tx * w / tw;
                    let a = tx as usize * 4 + ty as usize * tw as usize * 4;
                    thumb[a..a + 4].copy_from_slice(&row[sx as usize * 4..sx as usize * 4 + 4]);
                }
                ty += 1;
            }
            pixels.extend_from_slice(&row);
        }
        let p = stage.path().join(format!("tile-{}.png", tiles.len()));
        encode(&p, w, count, &pixels, "viewer tile v1")?;
        tiles.push(p);
        y += count;
    }
    rows.finish()?;
    check_cancel(cancel)?;
    let tp = stage.path().join("thumbnail.png");
    encode(&tp, tw, th, &thumb, "thumbnail v1")?;
    let original = store.put_artifact(fs::File::open(path)?, kind)?.id;
    let thumbnail = store
        .put_artifact(fs::File::open(tp)?, ArtifactKind::Thumbnail)?
        .id;
    let tiles = tiles
        .into_iter()
        .map(|p| {
            store
                .put_artifact(fs::File::open(p)?, ArtifactKind::Tile)
                .map(|a| a.id)
        })
        .collect::<Result<_>>()?;
    Ok(ImageEvidence {
        original,
        thumbnail,
        width: w,
        height: h,
        tiles,
    })
}
