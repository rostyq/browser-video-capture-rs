#[macro_export]
macro_rules! js_set {
    ($target:expr, $field:literal, $value:expr) => {
        web_sys::js_sys::Reflect::set(
            $target.as_ref(),
            &web_sys::wasm_bindgen::JsValue::from($field),
            &web_sys::wasm_bindgen::JsValue::from($value),
        )
        .unwrap()
        .then_some(())
        .unwrap();
    };
}

#[macro_export]
macro_rules! impl_canvas_capture_area {
    ($name:tt) => {
        impl crate::CaptureArea for $name {
            fn capture_width(&self) -> u32 {
                self.canvas.width()
            }

            fn capture_height(&self) -> u32 {
                self.canvas.height()
            }

            fn set_capture_width(&self, width: u32) {
                self.canvas.set_width(width);
            }

            fn set_capture_height(&self, height: u32) {
                self.canvas.set_height(height);
            }
        }
    };
}

#[macro_export]
macro_rules! impl_capture_2d {
    ($name:tt $canvas:tt $context:tt $options:tt) => {
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
                source: &HtmlVideoElement,
                mode: CaptureMode,
            ) -> Result<(u32, u32), js_sys::Error> {
                match mode {
                    CaptureMode::Put(dx, dy) => self
                        .context
                        .draw_image_with_html_video_element(source, dx as f64, dy as f64)
                        .map(|_| video_size(source)),
                    CaptureMode::Fill => {
                        let (dw, dh) = self.capture_size();

                        self.context
                            .draw_image_with_html_video_element_and_dw_and_dh(
                                source, 0.0, 0.0, dw as f64, dh as f64,
                            )
                            .map(|_| (dw, dh))
                    }
                    CaptureMode::Adjust => {
                        let (dw, dh) = self.capture_size();
                        let (sw, sh) = video_size(source);

                        if sw != dw || sh != dh {
                            self.set_capture_size(sw, sh);
                        }

                        self.context
                            .draw_image_with_html_video_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                                source, 0.0, 0.0, sw as f64, sh as f64, 0.0, 0.0, dw as f64, dh as f64,
                            )
                            .map(|_| (dw, dh))
                    }
                    CaptureMode::Pinhole => {
                        let (cw, ch) = self.capture_size();
                        let (sw, sh) = video_size(source);

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

            fn read(&self, source: &HtmlVideoElement, mode: CaptureMode) -> Result<Vec<u8>, js_sys::Error> {
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
