use std::fmt::Display;

use web_sys::{
    js_sys,
    wasm_bindgen::{JsCast, JsValue},
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlVideoElement,
};

use crate::{utils::video_size, BrowserVideoCapture, CaptureArea, CaptureMode};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HtmlContextOptions2D {
    pub alpha: bool,
    pub desynchronized: bool,
    pub will_read_frequently: bool,
    pub color_space: ColorSpaceType,
}

impl Into<JsValue> for HtmlContextOptions2D {
    fn into(self) -> JsValue {
        let options = js_sys::Object::new();
        js_set!(options, "alpha", self.alpha);
        js_set!(options, "desynchronized", self.desynchronized);
        js_set!(options, "willReadFrequently", self.will_read_frequently);
        js_set!(options, "colorSpace", self.color_space.to_string());
        options.into()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ColorSpaceType {
    #[default]
    Srgb,
    DisplayP3,
}

impl Display for ColorSpaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorSpaceType::Srgb => write!(f, "srgb"),
            ColorSpaceType::DisplayP3 => write!(f, "display-p3"),
        }
    }
}

impl_capture_2d!(HtmlCapture2D HtmlCanvasElement CanvasRenderingContext2d HtmlContextOptions2D);

impl HtmlCapture2D {
    pub fn new(context: CanvasRenderingContext2d) -> Option<Self> {
        context.canvas().map(|canvas| Self { context, canvas })
    }
}
