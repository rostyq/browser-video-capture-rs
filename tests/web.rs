//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;

use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

use web_sys::{
    js_sys::Promise, wasm_bindgen::JsCast, CanvasRenderingContext2d, HtmlCanvasElement,
    HtmlVideoElement, MediaStream, MediaStreamTrack, OffscreenCanvas,
};

use image::Rgba;

#[allow(unused_imports)]
use gloo::{
    console::{self, console_dbg},
    utils::{body, document, window},
};

use browser_video_capture::{
    BrowserVideoCapture, CaptureColor, CaptureMode, HtmlCapture2D, OffscreenCapture2D, OffscreenCaptureGL,
};

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

macro_rules! create_canvas {
    (html $w:expr,$h:expr) => {
        create_html_canvas($w, $h)
    };
    (offscreen $w:expr,$h:expr) => {
        create_offscreen_canvas($w, $h)
    };
}

macro_rules! create_capture {
    ($cv:tt $kind:ty, $color:expr, ($w:expr,$h:expr)) => {
        <$kind>::from_canvas(create_canvas!($cv $w,$h), $color)
            .unwrap()
            .unwrap()
            .validate()
            .unwrap()
    };
    [$($cv:tt $kind:ty, $color:expr, ($w:expr,$h:expr));*] => {
        {
            let mut arr = Vec::new();
            $(
                arr.push(Box::new(create_capture!($cv $kind, $color, ($w,$h))) as Box<dyn BrowserVideoCapture>);
            )*
            arr
        }
    };
}

macro_rules! create_video {
    () => {
        document()
            .create_element("video")
            .unwrap()
            .dyn_into::<HtmlVideoElement>()
            .unwrap()
    };
    ($width:expr, $height:expr) => {{
        let v = create_video!();
        v.set_width($width);
        v.set_height($height);
        v.set_muted(true);
        v.set_autoplay(true);
        v.toggle_attribute("playsinline").unwrap();
        v
    }};
    (pseudo_hidden) => {{
        let v = create_video!(1, 1);
        body().append_child(&v).unwrap();
        v
    }};
}

pub struct CaptureContext {
    pub video: HtmlVideoElement,
    pub canvas: HtmlCanvasElement,
    pub context: CanvasRenderingContext2d,
    pub stream: MediaStream,
}

impl CaptureContext {
    pub fn new(width: u32, height: u32) -> Self {
        let canvas = create_html_canvas(width, height);
        let stream = canvas.capture_stream().unwrap();

        body().append_child(&canvas).unwrap();

        let video = create_video!(pseudo_hidden);
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
}

impl Drop for CaptureContext {
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

#[wasm_bindgen_test]
fn capture_ignores_empty_video() {
    let source = create_video!();
    let caps = create_capture![
        html HtmlCapture2D, CaptureColor::RGBA, (DEFAULT_WIDTH, DEFAULT_HEIGHT);
        offscreen OffscreenCapture2D, CaptureColor::RGBA, (DEFAULT_WIDTH, DEFAULT_HEIGHT);
        offscreen OffscreenCaptureGL, CaptureColor::RGBA, (DEFAULT_WIDTH, DEFAULT_HEIGHT)
    ];

    for cap in caps.into_iter() {
        for mode in [
            CaptureMode::put_top_left(),
            CaptureMode::Fill,
            CaptureMode::Pinhole,
        ] {
            cap.capture(&source, mode);
            assert_eq!(cap.capture_width(), DEFAULT_WIDTH);
            assert_eq!(cap.capture_height(), DEFAULT_HEIGHT);

            let data = cap.data();
            assert_eq!(data.len(), cap.buffer_size());
            console_dbg!(&data[0..8]);
            for value in data.into_iter() {
                assert_eq!(value, 0);
            }
        }
    }
}

#[wasm_bindgen_test]
async fn all_modes_capture_video_frame_1x1() {
    let ctx = CaptureContext::new(1, 1);
    let caps = create_capture![
        html HtmlCapture2D, CaptureColor::RGBA, (1, 1);
        offscreen OffscreenCapture2D, CaptureColor::RGBA, (1, 1);
        offscreen OffscreenCaptureGL, CaptureColor::RGBA, (1, 1)
    ];

    ctx.context.set_fill_style(&"white".into());
    ctx.context
        .fill_rect(0.0, 0.0, DEFAULT_WIDTH as f64, DEFAULT_HEIGHT as f64);

    wait_next_frame(&ctx.video).await;

    for cap in caps.iter() {
        for mode in [
            CaptureMode::put_top_left(),
            CaptureMode::Fill,
            CaptureMode::Pinhole,
            CaptureMode::Adjust,
        ] {
            cap.capture(&ctx.video, mode);
            let data = cap.image().unwrap().into_rgba8();
            assert_eq!(data.len(), cap.buffer_size());
            assert_eq!(data.len(), 4);
            assert_eq!(data.get_pixel(0, 0), &Rgba([255, 255, 255, 255]));
        }
    }
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

fn create_html_canvas(width: u32, height: u32) -> HtmlCanvasElement {
    let e = document().create_element("canvas").unwrap();

    e.set_attribute("width", &width.to_string()).unwrap();
    e.set_attribute("height", &height.to_string()).unwrap();

    e.dyn_into().unwrap()
}

fn create_offscreen_canvas(width: u32, height: u32) -> OffscreenCanvas {
    OffscreenCanvas::new(width, height).unwrap()
}
