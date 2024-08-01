use std::fmt::Display;

use web_sys::{
    js_sys,
    wasm_bindgen::{JsCast, JsValue},
    HtmlVideoElement, OffscreenCanvas, OffscreenCanvasRenderingContext2d,
};

use crate::{utils::video_size, BrowserVideoCapture, CaptureArea, CaptureMode};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OffscreenContextOptions2D {
    pub alpha: bool,
    pub will_read_frequently: bool,
    pub storage: OffscreenStorageType,
}

impl Into<JsValue> for OffscreenContextOptions2D {
    fn into(self) -> JsValue {
        let options = js_sys::Object::new();
        js_set!(options, "alpha", self.alpha);
        js_set!(options, "willReadFrequently", self.will_read_frequently);
        js_set!(options, "storage", self.storage.to_string());
        options.into()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OffscreenStorageType {
    #[default]
    Persistent,
}

impl Display for OffscreenStorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OffscreenStorageType::Persistent => write!(f, "persistent"),
        }
    }
}

impl_capture_2d!(OffscreenCapture2D OffscreenCanvas OffscreenCanvasRenderingContext2d OffscreenContextOptions2D);

impl OffscreenCapture2D {
    pub fn new(context: OffscreenCanvasRenderingContext2d) -> Self {
        let canvas = context.canvas();
        Self { context, canvas }
    }
}
