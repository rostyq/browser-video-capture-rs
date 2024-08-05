#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;

#[cfg(test)]
pub(crate) mod wasm {
    pub use wasm_bindgen_test::wasm_bindgen_test as test;
}

use image::Rgba;
use rstest::*;
use wasm_bindgen_test::*;

use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use web_sys::{
    js_sys::Promise, wasm_bindgen::JsCast, CanvasRenderingContext2d, HtmlCanvasElement,
    HtmlVideoElement, MediaStream, MediaStreamTrack, OffscreenCanvas,
};

#[allow(unused_imports)]
use gloo::{
    console::{self, console_dbg},
    utils::{body, document, window},
};

use browser_video_capture::{
    impl_canvas_capture_area, BrowserCapture, BrowserCaptureBuilder, BrowserVideoCapture, CaptureArea, CaptureMode, GLVersion, HtmlContextOptions2D, HtmlContextOptionsGL, OffscreenContextOptions2D, OffscreenContextOptionsGL, SupportedCanvas, SupportedOptions
};

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

pub struct CaptureSetup {
    pub video: HtmlVideoElement,
    pub canvas: HtmlCanvasElement,
    pub context: CanvasRenderingContext2d,
    pub stream: MediaStream,
}

impl CaptureSetup {
    pub fn new(canvas: HtmlCanvasElement, video: HtmlVideoElement) -> Self {
        body().append_child(&canvas).unwrap();
        body().append_child(&video).unwrap();

        let stream = canvas.capture_stream().unwrap();
        video.set_src_object(Some(&stream));

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        Self {
            video,
            canvas,
            context,
            stream,
        }
    }

    pub fn from_size(source_width: u32, source_height: u32) -> Self {
        let video = video_source(source_width, source_height);
        let canvas = document()
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();
        canvas.set_width(source_width);
        canvas.set_height(source_height);
        Self::new(canvas, video)
    }
}

impl_canvas_capture_area!(CaptureSetup);

impl Drop for CaptureSetup {
    fn drop(&mut self) {
        self.stream
            .get_tracks()
            .iter()
            .for_each(|t| t.dyn_ref::<MediaStreamTrack>().unwrap().stop());
        self.video.set_src_object(None);
        body().remove_child(&self.video).unwrap();
        body().remove_child(&self.canvas).unwrap();
    }
}

wasm_bindgen_test_configure!(run_in_browser);

fn create_capture(width: u32, height: u32, options: SupportedOptions) -> BrowserCapture {
    BrowserCaptureBuilder::default()
        .canvas(capture_canvas(width, height, options))
        .options(options)
        .build()
        .unwrap()
        .unwrap()
}

#[rstest]
#[wasm::test]
fn capture_ignores_empty_video(
    #[values(
        CaptureSetup::from_size(0, 0),
        CaptureSetup::from_size(1, 1),
        CaptureSetup::from_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
    )]
    setup: CaptureSetup,
    #[values(
        HtmlContextOptions2D::default().alpha(true).will_read_frequently(true).into(),
        HtmlContextOptionsGL::default().alpha(true).into(),
        HtmlContextOptionsGL::default().alpha(true).version(GLVersion::WebGL2).into(),
        OffscreenContextOptions2D::default().alpha(true).will_read_frequently(true).into(),
        OffscreenContextOptionsGL::default().alpha(true).into(),
        OffscreenContextOptionsGL::default().alpha(true).version(GLVersion::WebGL2).into()
    )]
    options: SupportedOptions,
    #[values(
        CaptureMode::put_top_left(),
        CaptureMode::Fill,
        CaptureMode::Adjust,
        // CaptureMode::Pinhole
    )]
    mode: CaptureMode,
) {
    let cap = create_capture(DEFAULT_WIDTH, DEFAULT_HEIGHT, options);
    console_dbg!(cap);

    cap.capture(&setup.video, mode);
    assert_eq!(cap.capture_width(), DEFAULT_WIDTH);
    assert_eq!(cap.capture_height(), DEFAULT_HEIGHT);

    let data = cap.data();
    assert_eq!(data.len(), cap.buffer_size());
    for value in data.into_iter() {
        assert_eq!(value, 0);
    }
}

#[rstest]
#[wasm::test]
async fn capture_non_empty_video_same_size(
    #[values(
        CaptureSetup::from_size(1, 1),
        CaptureSetup::from_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
    )]
    setup: CaptureSetup,
    #[values(
        HtmlContextOptions2D::default().will_read_frequently(true).into(),
        HtmlContextOptionsGL::default().into(),
        HtmlContextOptionsGL::default().version(GLVersion::WebGL2).into(),
        OffscreenContextOptions2D::default().will_read_frequently(true).into(),
        OffscreenContextOptionsGL::default().into(),
        OffscreenContextOptionsGL::default().version(GLVersion::WebGL2).into()
    )]
    options: SupportedOptions,
    #[values(
        CaptureMode::put_top_left(),
        CaptureMode::Fill,
        CaptureMode::Adjust,
        // CaptureMode::Pinhole
    )]
    mode: CaptureMode,
) {
    let (width, height) = setup.capture_size();
    let cap = create_capture(width, height, options);

    setup.context.set_fill_style(&"white".into());
    setup
        .context
        .fill_rect(0.0, 0.0, width as f64, height as f64);
    wait_next_frame(&setup.video).await;

    cap.capture(&setup.video, mode);
    let data = cap.image().unwrap().into_rgba8();
    assert_eq!(data.len(), cap.buffer_size());
    assert_eq!(data.get_pixel(0, 0), &Rgba([255, 255, 255, 255]));
}

#[rstest]
#[wasm::test]
async fn capture_simple_four_color(
    #[values(
        CaptureSetup::from_size(4, 4),
        CaptureSetup::from_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
    )]
    setup: CaptureSetup,
    #[values(
        HtmlContextOptions2D::default().will_read_frequently(true).into(),
        OffscreenContextOptions2D::default().will_read_frequently(true).into(),
        OffscreenContextOptionsGL::default().into()
    )]
    options: SupportedOptions,
    #[values(
        CaptureMode::put_top_left(),
        CaptureMode::Fill,
        CaptureMode::Adjust,
        // CaptureMode::Pinhole,
    )]
    mode: CaptureMode,
) {
    let (w, h) = setup.capture_size();
    let cap = create_capture(w, h, options);
    console_dbg!((w, h));
    console_dbg!(cap);

    // let (w, h) = (width as f64, height as f64);
    let (x, y) = ((w / 2) as f64, (h / 2) as f64);

    setup.context.set_fill_style(&"rgb(255, 0, 0)".into());
    setup.context.fill_rect(0.0, 0.0, x, y);

    setup.context.set_fill_style(&"rgb(0, 255, 0)".into());
    setup.context.fill_rect(x, 0.0, x, y);

    setup.context.set_fill_style(&"rgb(0, 0, 255)".into());
    setup.context.fill_rect(0.0, y, x, y);

    setup.context.set_fill_style(&"rgb(255, 255, 255)".into());
    setup.context.fill_rect(x, y, x, y);

    wait_next_frame(&setup.video).await;

    cap.capture(&setup.video, mode);
    let data = cap.image().unwrap().into_rgba8();

    let (r, b) = (w - 1, h - 1);
    assert_eq!(data.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
    assert_eq!(data.get_pixel(r, 0), &Rgba([0, 255, 0, 255]));
    assert_eq!(data.get_pixel(0, b), &Rgba([0, 0, 255, 255]));
    assert_eq!(data.get_pixel(r, b), &Rgba([255, 255, 255, 255]));
}

fn animation_frame() -> JsFuture {
    Promise::new(&mut |resolve, reject| {
        if let Err(value) = window().request_animation_frame(&resolve) {
            reject.call1(&JsValue::undefined(), &value).unwrap();
        }
    })
    .into()
}

async fn wait_next_frame(video: &HtmlVideoElement) {
    let t1 = video.current_time();
    loop {
        animation_frame().await.unwrap();
        let t2 = video.current_time();

        if t2 > t1 {
            break;
        }
    }
}

fn video_source(width: u32, height: u32) -> HtmlVideoElement {
    let e = document()
        .create_element("video")
        .unwrap()
        .dyn_into::<HtmlVideoElement>()
        .unwrap();
    e.set_width(width);
    e.set_height(height);
    e.set_muted(true);
    e.set_autoplay(true);
    e.toggle_attribute("playsinline").unwrap();
    e
}

fn capture_canvas(width: u32, height: u32, options: SupportedOptions) -> SupportedCanvas {
    match options {
        SupportedOptions::Html2D(_) | SupportedOptions::HtmlGL(_) => SupportedCanvas::Html({
            let e = document()
                .create_element("canvas")
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .unwrap();
            e.set_width(width);
            e.set_height(height);
            e
        }),
        SupportedOptions::Offscreen2D(_) | SupportedOptions::OffscreenGL(_) => {
            SupportedCanvas::Offscreen(OffscreenCanvas::new(width, height).unwrap())
        }
    }
}
