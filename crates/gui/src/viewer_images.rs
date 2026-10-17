// SPDX-License-Identifier: MPL-2.0
use anyhow::{Context, Result};
use regressee_core::domain::ArtifactId;
use regressee_storage::Store;
use std::sync::Arc;

pub fn viewer_tile(store: &Store, id: &ArtifactId) -> Result<Arc<gpui::RenderImage>> {
    Ok(viewer_tile_with_size(store, id)?.0)
}

pub(crate) fn viewer_tile_with_size(
    store: &Store,
    id: &ArtifactId,
) -> Result<(Arc<gpui::RenderImage>, usize)> {
    let (width, height, mut pixels) = regressee_app::images::decode_tile(store, id)?;
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
    }
    let size = pixels.len();
    let buffer =
        image::RgbaImage::from_raw(width, height, pixels).context("invalid viewer tile pixels")?;
    Ok((
        Arc::new(gpui::RenderImage::new(vec![image::Frame::new(buffer)])),
        size,
    ))
}
