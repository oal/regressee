// SPDX-License-Identifier: MPL-2.0
//! Embedded, background-appropriate logos paired with a readable product name.
use super::theme::{INK, RAIL_INK};
use gpui::{Div, FontWeight, Image, ImageFormat, div, img, prelude::*, px, rgb};
use std::sync::{Arc, OnceLock};

const LIGHT_BYTES: &[u8] = include_bytes!("../../../../assets/brand/logo-light.png");
const DARK_BYTES: &[u8] = include_bytes!("../../../../assets/brand/logo-dark.png");

#[derive(Clone, Copy)]
pub enum Background {
    Light,
    Dark,
}

/// Stable image identity lets GPUI decode/cache the embedded PNG off-thread.
/// No filesystem lookup, network fetch, or image decoding in the render callback.
pub fn logo(background: Background) -> Arc<Image> {
    static LIGHT: OnceLock<Arc<Image>> = OnceLock::new();
    static DARK: OnceLock<Arc<Image>> = OnceLock::new();
    let (cached, bytes) = match background {
        Background::Light => (&LIGHT, LIGHT_BYTES),
        Background::Dark => (&DARK, DARK_BYTES),
    };
    cached
        .get_or_init(|| Arc::new(Image::from_bytes(ImageFormat::Png, bytes.to_vec())))
        .clone()
}

/// Noninteractive identity: the image supplements, never replaces, the name.
pub fn lockup(background: Background, label: &'static str) -> Div {
    div()
        .debug_selector(|| "brand-lockup".into())
        .flex()
        .items_center()
        .gap(px(8.))
        .flex_shrink_0()
        .text_color(rgb(match background {
            Background::Light => INK,
            Background::Dark => RAIL_INK,
        }))
        .font_weight(FontWeight::SEMIBOLD)
        .child(
            div()
                .debug_selector(|| "brand-logo".into())
                .w(px(36.))
                .h(px(24.))
                .flex_shrink_0()
                .child(img(logo(background)).w(px(36.)).h(px(36. * 180. / 294.))),
        )
        .child(
            div()
                .debug_selector(|| "brand-name".into())
                .flex_shrink_0()
                .child(label),
        )
}

/// The rail gives the artwork its own row so the comparison-eye detail is legible.
pub fn navigation_lockup() -> Div {
    const LOGO_WIDTH: f32 = 96.;
    const LOGO_HEIGHT: f32 = LOGO_WIDTH * 180. / 294.;
    div()
        .debug_selector(|| "brand-lockup".into())
        .w_full()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.))
        .flex_shrink_0()
        .text_color(rgb(RAIL_INK))
        .text_size(px(20.))
        .line_height(px(26.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(
            div()
                .debug_selector(|| "brand-nav-logo".into())
                .w(px(LOGO_WIDTH))
                .h(px(LOGO_HEIGHT))
                .flex_shrink_0()
                .child(
                    img(logo(Background::Dark))
                        .w(px(LOGO_WIDTH))
                        .h(px(LOGO_HEIGHT)),
                ),
        )
        .child(
            div()
                .debug_selector(|| "brand-nav-name".into())
                .flex_shrink_0()
                .child("Regressee"),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logos_are_distinct_embedded_transparent_pngs_with_stable_cache_identity() {
        for background in [Background::Light, Background::Dark] {
            let image = logo(background);
            assert!(Arc::ptr_eq(&image, &logo(background)));
            let decoder = png::Decoder::new(std::io::Cursor::new(&image.bytes));
            let reader = decoder.read_info().unwrap();
            assert_eq!((reader.info().width, reader.info().height), (294, 180));
            assert_eq!(reader.info().color_type, png::ColorType::Rgba);
        }
        assert_ne!(logo(Background::Light).id(), logo(Background::Dark).id());
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod ui_tests {
    use super::*;
    use gpui::{Context, Render, TestAppContext, Window, size};

    struct NavigationBrand;

    impl Render for NavigationBrand {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(192.))
                .p_3()
                .child(div().px_2().pt_3().pb_3().child(navigation_lockup()))
        }
    }

    #[gpui::test]
    fn navigation_mark_is_larger_centered_and_above_the_name(cx: &mut TestAppContext) {
        for (width, height) in [(1280., 800.), (820., 600.)] {
            let (_, cx) = cx.add_window_view(|_, _| NavigationBrand);
            cx.simulate_resize(size(px(width), px(height)));
            let mark = cx.debug_bounds("brand-nav-logo").unwrap();
            let name = cx.debug_bounds("brand-nav-name").unwrap();
            assert!((f32::from(mark.size.width) - 96.).abs() < 0.1);
            // GPUI rounds layout boundaries to device pixels.
            assert!((f32::from(mark.size.height) - 96. * 180. / 294.).abs() < 1.);
            assert!((f32::from(mark.center().x) - 96.).abs() < 0.1);
            assert!((f32::from(name.center().x) - 96.).abs() < 0.1);
            assert!((f32::from(name.top() - mark.bottom()) - 8.).abs() < 0.1);
            assert!(mark.left() >= px(20.) && mark.right() <= px(172.));
            assert!(name.left() >= px(20.) && name.right() <= px(172.));
        }
    }
}
