use std::fmt::Display;

use web_sys::{
    js_sys,
    wasm_bindgen::{JsCast, JsValue},
};

use crate::{BrowserVideoCapture, CaptureArea};

macro_rules! impl_capture_2d {
    ($name:tt $canvas:ty, $context:ty, $options:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            canvas: $canvas,
            context: $context,
            color: crate::CaptureColor,
        }

        impl $name {
            pub fn new(canvas: $canvas, context: $context, color: crate::CaptureColor) -> Self {
                Self { canvas, context, color }
            }

            pub fn validate(self) -> Result<Self, Option<String>> {
                Ok(self)
            }

            fn read_data(&self, x: i32, y: i32, width: u32, height: u32) -> Vec<u8> {
                let image_data =
                    self.context
                        .get_image_data(x as f64, y as f64, width as f64, height as f64).unwrap();
                image_data.data().0
            }
        }

        impl_capture_from_canvas!("2d", $name, $canvas, $context, $options);
        impl_canvas_capture_area!($name);

        impl BrowserVideoCapture for $name {
            fn capture(
                &self,
                source: &web_sys::HtmlVideoElement,
                mode: crate::CaptureMode,
            ) -> (u32, u32) {
                let (sw, sh) = crate::utils::video_size(source);
                let (mut cw, mut ch) = self.capture_size();

                if sw == 0 || sh == 0 {
                    return (cw, ch);
                }

                self.context.set_image_smoothing_enabled(false);

                match mode {
                    crate::CaptureMode::Put(dx, dy) => {
                        if dx > 0 || dy > 0 {
                            self.clear();
                        } else {
                            let (sw, sh) = crate::utils::video_size(source);

                            if (sw as i32 - dx) < cw as i32 || (sh as i32 - dy) < ch as i32 {
                                self.clear();
                            }
                        }

                        self.context.draw_image_with_html_video_element(source, dx as f64, dy as f64)
                    },
                    crate::CaptureMode::Fill => {
                        self.context
                            .draw_image_with_html_video_element_and_dw_and_dh(
                                source, 0.0, 0.0, cw as f64, ch as f64,
                            )
                    }
                    crate::CaptureMode::Adjust => {
                        if sw != cw || sh != ch {
                            self.set_capture_size(sw, sh);
                            cw = sw;
                            ch = sh;
                        }

                        self.context
                            .draw_image_with_html_video_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                                source, 0.0, 0.0, sw as f64, sh as f64, 0.0, 0.0, cw as f64, ch as f64,
                            )
                    }
                    crate::CaptureMode::Pinhole => {
                        if sw < cw || sh < ch {
                            self.clear();
                        }

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
                    }
                }.unwrap();

                (cw, ch)
            }

            fn retrieve(&self, buffer: &mut [u8]) {
                let (w, h) = self.capture_size();
                if w > 0 && h > 0 {
                    let data = self.read_data(0, 0, w, h);
                    buffer.copy_from_slice(data.as_slice());
                }
            }

            fn data(&self) -> Vec<u8> {
                let (w, h) = self.capture_size();
                if w > 0 && h > 0 {
                    self.read_data(0, 0, w, h)
                } else {
                    Vec::new()
                }
            }

            fn read(&self, source: &web_sys::HtmlVideoElement, mode: crate::CaptureMode) -> Vec<u8> {
                let (w, h) = self.capture(source, mode);
                if w > 0 && h > 0 {
                    self.read_data(0, 0, w, h)
                } else {
                    Vec::new()
                }
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

    impl_context_options!(
        HtmlContextOptions2D
        "alpha" alpha: bool,
        "desynchronized" desynchronized: bool,
        "willReadFrequently" will_read_frequently: bool,
        "colorSpace" color_space: ColorSpaceType
    );

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

    impl Into<JsValue> for ColorSpaceType {
        fn into(self) -> JsValue {
            JsValue::from(self.to_string())
        }
    }

    impl_capture_2d!(
        HtmlCapture2D
        web_sys::HtmlCanvasElement,
        web_sys::CanvasRenderingContext2d,
        HtmlContextOptions2D
    );

    impl HtmlCapture2D {
        pub fn from_context(
            context: web_sys::CanvasRenderingContext2d,
            color: crate::CaptureColor,
        ) -> Option<Self> {
            context.canvas().map(|canvas| Self {
                context,
                canvas,
                color,
            })
        }
    }
}

#[cfg(feature = "offscreen")]
pub mod offscreen {
    use super::*;

    impl_context_options!(
        OffscreenContextOptions2D
        "alpha" alpha: bool,
        "willReadFrequently" will_read_frequently: bool,
        "storage" storage: OffscreenStorageType
    );

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

    impl Into<JsValue> for OffscreenStorageType {
        fn into(self) -> JsValue {
            JsValue::from(self.to_string())
        }
    }

    impl_capture_2d!(
        OffscreenCapture2D
        web_sys::OffscreenCanvas,
        web_sys::OffscreenCanvasRenderingContext2d,
        OffscreenContextOptions2D
    );

    impl OffscreenCapture2D {
        pub fn from_context(
            context: web_sys::OffscreenCanvasRenderingContext2d,
            color: crate::CaptureColor,
        ) -> Self {
            let canvas = context.canvas();
            Self {
                context,
                canvas,
                color,
            }
        }
    }
}
