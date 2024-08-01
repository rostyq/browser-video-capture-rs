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
