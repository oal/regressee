use gpui::{actions, prelude::*, *};
// SPDX-License-Identifier: MPL-2.0
use regressee_gui::ui::text_input;
use regressee_spike::{
    actor::{Controller, Update, Work},
    images::Tiles,
};
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::Ordering, mpsc::Receiver},
    time::Duration,
};
actions!(bench, [Quit, Tab, TabPrev, OpenImage, Cancel, Activate]);
struct Bench {
    input: Entity<text_input::TextInput>,
    focus: FocusHandle,
    buttons: Vec<FocusHandle>,
    controller: Arc<Controller>,
    rx: Receiver<Update>,
    status: String,
    busy: bool,
    tiles: Option<Tiles>,
    tile_index: usize,
    cache: Entity<RetainAllImageCache>,
    clear_cache: bool,
}
impl Bench {
    fn new(cx: &mut Context<Self>, controller: Arc<Controller>, rx: Receiver<Update>) -> Self {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                if this
                    .update(cx, |this, cx| {
                        while let Ok(event) = this.rx.try_recv() {
                            this.status = event.status;
                            this.busy = event.busy;
                            if let Some(t) = event.tiles {
                                this.tiles = Some(t);
                                this.tile_index = 0;
                                this.clear_cache = true;
                            }
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        Self {
            input: cx.new(|cx| text_input::TextInput::with_value("/public", 0, cx)),
            focus: cx.focus_handle(),
            buttons: (0..8)
                .map(|i| cx.focus_handle().tab_index(i + 1).tab_stop(true))
                .collect(),
            controller,
            rx,
            status:
                "Phase 0 • local fixtures only. Build the development runtime pack before Repair."
                    .into(),
            busy: false,
            tiles: None,
            tile_index: 0,
            cache: RetainAllImageCache::new(cx),
            clear_cache: false,
        }
    }
    fn queue(&mut self, w: Work, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        match self.controller.activate(w) {
            Ok(()) => {
                self.busy = true;
                self.status = "Starting…".into();
                if std::env::var("REGRESSEE_UI_TEST").as_deref() == Ok("1") {
                    // A native-input test must observe actual activation, not
                    // mistake idle Escape acknowledgement for active cancel.
                    let _ = std::fs::write(
                        "artifacts/ui-activation.json.partial",
                        b"{\"queued\":true}",
                    );
                    let _ = std::fs::rename(
                        "artifacts/ui-activation.json.partial",
                        "artifacts/ui-activation.json",
                    );
                }
            }
            Err(_) => self.status = "Worker queue unavailable. Restart the spike.".into(),
        }
        cx.notify();
    }
    fn cancel(&mut self, _: &Cancel, _: &mut Window, cx: &mut Context<Self>) {
        self.controller.cancel.store(true, Ordering::Relaxed);
        self.busy = true;
        self.status = "Stopping capture and managed browser…".into();
        cx.notify();
    }
    fn open_image(&mut self, _: &OpenImage, _: &mut Window, cx: &mut Context<Self>) {
        let pick = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open a PNG screenshot".into()),
        });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(paths))) = pick.await
                && let Some(path) = paths.first()
            {
                let _ = this.update(cx, |this, cx| this.queue(Work::Image(path.clone()), cx));
            }
        })
        .detach();
    }
    fn action(&mut self, index: usize, w: &mut Window, cx: &mut Context<Self>) {
        match index {
            0 => self.queue(Work::Repair, cx),
            1 => self.queue(Work::Command("openLogin", json!({})), cx),
            2 => self.queue(Work::Command("finishLogin", json!({})), cx),
            3 => {
                let target = self.input.read(cx).content.to_string();
                self.queue(Work::Command("capture", json!({"target":target})), cx);
            }
            4 => self.queue(Work::Command("capture", json!({"target":"/tall"})), cx),
            5 => self.open_image(&OpenImage, w, cx),
            6 => self.cancel(&Cancel, w, cx),
            7 => {
                self.status="Menu: use Ctrl/Cmd+O to open a PNG; Escape cancels; Ctrl/Cmd+Q quits. Native OS menus are also registered.".into();
                cx.notify();
            }
            _ => (),
        }
    }
    fn activate(&mut self, _: &Activate, w: &mut Window, cx: &mut Context<Self>) {
        if let Some(i) = self.buttons.iter().position(|f| f.is_focused(w)) {
            self.action(i, w, cx);
        }
    }
}
impl Drop for Bench {
    fn drop(&mut self) {
        self.controller.stop();
    }
}
impl Render for Bench {
    fn render(&mut self, w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.clear_cache {
            self.cache.update(cx, |c, cx| c.clear(w, cx));
            self.clear_cache = false;
        }
        let labels = [
            "Repair runtime",
            "Open sign in",
            "I’m ready",
            "Capture address",
            "Capture tall page",
            "Open PNG…",
            "Cancel",
            "Menu",
        ];
        let tiles = self.tiles.clone();
        let index = self.tile_index;
        div().id("bench").track_focus(&self.focus).on_action(cx.listener(|_,_:&Tab,w,_|w.focus_next())).on_action(cx.listener(|_,_:&TabPrev,w,_|w.focus_prev())).on_action(cx.listener(Self::open_image)).on_action(cx.listener(Self::cancel)).on_action(cx.listener(Self::activate)).size_full().bg(rgb(0xf1f5f9)).text_color(rgb(0x172b40)).flex().flex_col().p_5().gap_3()
            .child(div().text_2xl().child("Regressee feasibility bench"))
            .child(div().text_sm().child("Synthetic local website · ephemeral login · pinned Chromium · Phase 0 only"))
            .child(div().flex().flex_wrap().gap_2().children(labels.into_iter().enumerate().map(|(i,label)|div().id(("button",i)).track_focus(&self.buttons[i]).px_3().py_2().border_1().rounded_md().border_color(rgb(0x64748b)).bg(rgb(if i==3{0x1d4ed8}else{0xffffff})).text_color(rgb(if i==3{0xffffff}else{0x172b40})).focus(|s|s.border_2().border_color(rgb(0x1d4ed8))).hover(|s|s.bg(rgb(0xdbeafe)).text_color(rgb(0x172b40))).when(self.busy&&i!=6,|s|s.opacity(0.5)).cursor_pointer().child(label).on_click(cx.listener(move|this,_,w,cx|this.action(i,w,cx))))))
            .child(div().child("Fixture address: /public or /private (sign in first)"))
            .child(self.input.clone())
            .child(div().text_sm().child(self.status.clone()))
            .child(div().image_cache(self.cache.clone()).id("viewer").flex_1().min_h_0().overflow_y_scroll().bg(rgb(0xffffff)).border_1().border_color(rgb(0x94a3b8)).child(match tiles{
                None=>div().p_8().child("Capture a page or open a PNG to inspect it. Tall images are never uploaded as a single texture.").into_any_element(),
                Some(t)=>div().flex().flex_col().child(div().p_2().child(format!("{} × {} pixels • tile {} of {} • scroll for the next tile • at most one texture retained",t.width,t.height,index+1,t.files.len()))).child(img(PathBuf::from(&t.files[index])).w(px(720.)).h(px((512u32.min(t.height-index as u32*512)) as f32*720./t.width as f32))).into_any_element()
            }).on_scroll_wheel(cx.listener(|this,e:&ScrollWheelEvent,w,cx|{if let Some(t)=&this.tiles{let delta=e.delta.pixel_delta(px(20.));let next=if delta.y<px(0.){(this.tile_index+1).min(t.files.len()-1)}else{this.tile_index.saturating_sub(1)};if next!=this.tile_index{this.cache.update(cx,|c,cx|c.clear(w,cx));this.tile_index=next;cx.notify();}}})))
    }
}
fn main() {
    let (controller, rx) = Controller::start(
        ".runtime/installed".into(),
        ".runtime/pack.zip".into(),
        "artifacts/ui-tiles".into(),
    );
    let after_run = controller.clone();
    Application::new().run(move |cx: &mut App| {
        text_input::bind(cx);
        cx.bind_keys([
            KeyBinding::new("tab", Tab, None),
            KeyBinding::new("shift-tab", TabPrev, None),
            KeyBinding::new("enter", Activate, None),
            KeyBinding::new("space", Activate, None),
            KeyBinding::new("ctrl-o", OpenImage, None),
            KeyBinding::new("cmd-o", OpenImage, None),
            KeyBinding::new("escape", Cancel, None),
            KeyBinding::new("ctrl-q", Quit, None),
            KeyBinding::new("cmd-q", Quit, None),
        ]);
        let p = controller.clone();
        cx.on_action(move |_: &Quit, cx| {
            p.stop();
            cx.quit();
        });
        cx.set_menus(vec![Menu {
            name: "Regressee".into(),
            items: vec![
                MenuItem::action("Open PNG…", OpenImage),
                MenuItem::action("Cancel capture", Cancel),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ],
        }]);
        let p = controller.clone();
        cx.on_window_closed(move |cx| {
            if cx.windows().is_empty() {
                p.stop();
                cx.quit();
            }
        })
        .detach();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1120.), px(800.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Regressee — Phase 0".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |w, cx| {
                let b = cx.new(|cx| Bench::new(cx, controller, rx));
                w.focus(&b.read(cx).input.read(cx).focus);
                b
            },
        )
        .unwrap();
        cx.activate(true);
    });
    after_run.stop();
}
