#[macro_export]
macro_rules! js_set {
    ($target:expr, $field:literal, $value:expr) => {
        web_sys::js_sys::Reflect::set(
            $target.as_ref(),
            &web_sys::wasm_bindgen::JsValue::from($field),
            &$value.into(),
        )
        .unwrap()
        .then_some(())
        .unwrap();
    };
}

#[macro_export]
macro_rules! impl_canvas_capture_area {
    ($name:ty) => {
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
macro_rules! get_context {
    ("2d" $name:literal, $canvas:expr, $options:expr) => {
        $canvas.get_context_with_context_options("2d", &$options.into())
    };
    ("webgl" $name:literal, $canvas:expr, $options:expr) => {
        $canvas.get_context_with_context_options(
            $options
                .experimental
                .then_some(concat!($name, "-experimental"))
                .unwrap_or($name),
            &$options.into(),
        )
    };
    ("webgl2" $name:literal, $canvas:expr, $options:expr) => {
        $canvas.get_context_with_context_options(
            $options
                .experimental
                .then_some(concat!($name, "-experimental"))
                .unwrap_or($name),
            &$options.into(),
        )
    };
}

#[macro_export]
macro_rules! impl_capture_from_canvas {
    ($id:tt, $capture:ty, $canvas:ty, $context:ty, $option:ty) => {
        impl $capture {
            pub fn from_canvas(canvas: $canvas, color: crate::CaptureColor) -> Result<Option<Self>, js_sys::Error> {
                canvas
                    .get_context($id)
                    .map(|value| {
                        value
                            .map(|obj| obj.dyn_into::<$context>().unwrap())
                            .map(|context| Self::new(canvas, context, color))
                    })
                    .map_err(|value| value.into())
            }

            pub fn from_canvas_with_options(
                canvas: $canvas,
                color: crate::CaptureColor,
                options: $option,
            ) -> Result<Option<Self>, js_sys::Error> {
                get_context!($id $id, canvas, options)
                    .map(|value| {
                        value
                            .map(|obj| obj.dyn_into::<$context>().unwrap())
                            .map(|context| Self::new(canvas, context, color))
                    })
                    .map_err(|value| value.into())
            }
        }
    };
}

#[macro_export]
macro_rules! options_field {
    ("" $obj:expr, $alias:literal, $value:expr) => {
    };

    ($field:tt $obj:expr, $alias:literal, $value:expr) => {
        js_set!($obj, $alias, $value);
    };
}

#[macro_export]
macro_rules! impl_context_options {
    ($name:tt $($alias:literal $field:tt: $typ:ty),+) => {
        #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            $(pub $field: $typ,)+
        }

        impl $name {
            $(pub fn $field(mut self, value: $typ) -> Self {
                self.$field = value;
                self
            })+
        }

        impl Into<JsValue> for $name {
            fn into(self) -> JsValue {
                let options = js_sys::Object::new();
                $(options_field!($alias options, $alias, self.$field);)+
                options.into()
            }
        }
    };
}
