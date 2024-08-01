use std::fmt::Display;

use web_sys::{
    js_sys,
    wasm_bindgen::{JsCast, JsValue},
};

use crate::{BrowserVideoCapture, CaptureArea};

#[cfg(feature = "html")]
pub mod html {
    use super::*;

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

    impl_capture_2d!(
        HtmlCapture2D
        web_sys::HtmlCanvasElement,
        web_sys::CanvasRenderingContext2d,
        HtmlContextOptions2D
    );

    impl HtmlCapture2D {
        pub fn new(context: web_sys::CanvasRenderingContext2d) -> Option<Self> {
            context.canvas().map(|canvas| Self { context, canvas })
        }
    }
}

#[cfg(feature = "offscreen")]
pub mod offscreen {
    use super::*;

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

    impl_capture_2d!(
        OffscreenCapture2D
        web_sys::OffscreenCanvas,
        web_sys::OffscreenCanvasRenderingContext2d,
        OffscreenContextOptions2D
    );

    impl OffscreenCapture2D {
        pub fn new(context: web_sys::OffscreenCanvasRenderingContext2d) -> Self {
            let canvas = context.canvas();
            Self { context, canvas }
        }
    }
}
