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
