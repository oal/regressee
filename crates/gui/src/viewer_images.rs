// SPDX-License-Identifier: MPL-2.0
use anyhow::{Context, Result};
#[cfg(any(test, feature = "test-support"))]
use regressee_core::domain::ArtifactId;
#[cfg(any(test, feature = "test-support"))]
use regressee_storage::Store;
use std::sync::Arc;

#[cfg(any(test, feature = "test-support"))]
pub fn viewer_tile(store: &Store, id: &ArtifactId) -> Result<Arc<gpui::RenderImage>> {
    Ok(viewer_tile_with_size(store, id)?.0)
}
pub fn from_artifact(
    file: &regressee_app::protocol::ArtifactFile,
) -> Result<Arc<gpui::RenderImage>> {
    let (width, height, mut pixels) = regressee_app::images::decode_tile_file(file)?;
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
    }
    let buffer =
        image::RgbaImage::from_raw(width, height, pixels).context("invalid viewer tile pixels")?;
    Ok(Arc::new(gpui::RenderImage::new(vec![image::Frame::new(
        buffer,
    )])))
}

#[cfg(any(test, feature = "test-support"))]
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
