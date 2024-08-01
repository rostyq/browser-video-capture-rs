use std::fmt::Display;

use web_sys::{
    js_sys,
    wasm_bindgen::{JsCast, JsValue},
};

use crate::{BrowserVideoCapture, CaptureArea};

macro_rules! impl_capture_2d {
    ($name:tt $canvas:path, $context:path, $options:path) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            canvas: $canvas,
            context: $context,
        }

        impl $name {
            pub fn from_canvas_with_options(
                canvas: $canvas,
                options: $options,
            ) -> Result<Option<Self>, js_sys::Error> {
                match canvas.get_context_with_context_options("2d", &options.into())? {
                    Some(obj) => {
                        let context = obj.dyn_into::<$context>().unwrap();
                        Ok(Some(Self { canvas, context }))
                    }
                    None => Ok(None),
                }
            }

            pub fn from_canvas(canvas: $canvas) -> Result<Option<Self>, js_sys::Error> {
                match canvas.get_context("2d")? {
                    Some(obj) => {
                        let context = obj.dyn_into::<$context>().unwrap();
                        Ok(Some(Self { canvas, context }))
                    }
                    None => Ok(None),
                }
            }

            fn read_data(&self, x: i32, y: i32, width: u32, height: u32) -> Result<Vec<u8>, js_sys::Error> {
                let image_data =
                    self.context
                        .get_image_data(x as f64, y as f64, width as f64, height as f64)?;
                Ok(image_data.data().0)
            }
        }

        impl_canvas_capture_area!($name);

        impl BrowserVideoCapture for $name {
            fn capture(
                &self,
                source: &web_sys::HtmlVideoElement,
                mode: crate::CaptureMode,
            ) -> Result<(u32, u32), js_sys::Error> {
                match mode {
                    crate::CaptureMode::Put(dx, dy) => self
                        .context
                        .draw_image_with_html_video_element(source, dx as f64, dy as f64)
                        .map(|_| crate::utils::video_size(source)),
                    crate::CaptureMode::Fill => {
                        let (dw, dh) = self.capture_size();

                        self.context
                            .draw_image_with_html_video_element_and_dw_and_dh(
                                source, 0.0, 0.0, dw as f64, dh as f64,
                            )
                            .map(|_| (dw, dh))
                    }
                    crate::CaptureMode::Adjust => {
                        let (dw, dh) = self.capture_size();
                        let (sw, sh) = crate::utils::video_size(source);

                        if sw != dw || sh != dh {
                            self.set_capture_size(sw, sh);
                        }

                        self.context
                            .draw_image_with_html_video_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                                source, 0.0, 0.0, sw as f64, sh as f64, 0.0, 0.0, dw as f64, dh as f64,
                            )
                            .map(|_| (dw, dh))
                    }
                    crate::CaptureMode::Pinhole => {
                        let (cw, ch) = self.capture_size();
                        let (sw, sh) = crate::utils::video_size(source);

                        let (dx, dy, dw, dh) = if sw > sh {
                            let dh = ch as f64 * sw as f64 / sh as f64;
                            ((cw as f64 - dh) / 2.0, 0.0, dh, dh)
                        } else {
                            let dw = cw as f64 * sh as f64 / sw as f64;
                            (0.0, (ch as f64 - dw) / 2.0, dw, dw)
                        };

                        self.context
                            .draw_image_with_html_video_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                                source, 0.0, 0.0, sw as f64, sh as f64, dx, dy, dw, dh,
                            )
                            .map(|_| (cw, ch))
                    }
                }
                .map_err(|value| value.dyn_into::<js_sys::Error>().unwrap())
            }

            fn retrieve(&self, buffer: &mut [u8]) -> Result<(), js_sys::Error> {
                let data = self.read_data(0, 0, self.capture_width(), self.capture_height())?;
                buffer.copy_from_slice(data.as_slice());
                Ok(())
            }

            fn data(&self) -> Result<Vec<u8>, js_sys::Error> {
                self.read_data(0, 0, self.capture_width(), self.capture_height())
            }

            fn read(&self, source: &web_sys::HtmlVideoElement, mode: crate::CaptureMode) -> Result<Vec<u8>, js_sys::Error> {
                let (width, height) = self.capture(source, mode)?;
                self.read_data(0, 0, width, height)
            }

            fn clear(&self) {
                self.context.clear_rect(
                    0.0,
                    0.0,
                    self.capture_width() as f64,
                    self.capture_height() as f64,
                );
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                self.clear();
            }
        }
    };
}

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
