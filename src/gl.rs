use std::fmt::Display;

use web_sys::{
    js_sys::{self, Float32Array, Uint16Array},
    wasm_bindgen::{JsCast, JsValue},
    WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture, WebGlUniformLocation,
};

use crate::{BrowserVideoCapture, CaptureArea};

macro_rules! initialize {
    (shader $gl:expr, $kind:expr, $src:expr) => {{
        $gl.create_shader($kind).map(|shader| {
            $gl.shader_source(&shader, $src);
            $gl.compile_shader(&shader);
            shader
        })
    }};
    (program $gl:expr, $vertex:expr, $fragment:expr) => {{
        $gl.create_program().map(|program| {
            $gl.attach_shader(&program, $vertex);
            $gl.attach_shader(&program, $fragment);
            $gl.link_program(&program);
            program
        })
    }};
    ($context:tt texture $gl:expr) => {{
        $gl.tex_parameteri(
            $context::TEXTURE_2D,
            $context::TEXTURE_WRAP_S,
            $context::CLAMP_TO_EDGE as i32,
        );
        $gl.tex_parameteri(
            $context::TEXTURE_2D,
            $context::TEXTURE_WRAP_T,
            $context::CLAMP_TO_EDGE as i32,
        );
        $gl.tex_parameteri(
            $context::TEXTURE_2D,
            $context::TEXTURE_MIN_FILTER,
            $context::LINEAR as i32,
        );
        $gl.tex_parameteri(
            $context::TEXTURE_2D,
            $context::TEXTURE_MAG_FILTER,
            $context::NEAREST as i32,
        );
    }};
}

macro_rules! validate {
    ($context:tt shader $gl:expr, $shader:expr) => {
        $gl.get_shader_parameter($shader, $context::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
            .then_some(())
            .ok_or_else(|| $gl.get_shader_info_log($shader))
    };
    ($context:tt program $gl:expr, $program:expr) => {
        $gl.get_program_parameter($program, $context::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
            .then_some(())
            .ok_or_else(|| $gl.get_program_info_log($program))
    };
}

macro_rules! impl_capture_gl {
    ($name:tt $canvas:ty, $context:tt, $options:ty, $capture_method:tt, $version:tt) => {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct $name {
        canvas: $canvas,
        context: $context,
        #[allow(dead_code)]
        color: crate::CaptureColor,

        vertex: Option<WebGlShader>,
        fragment: Option<WebGlShader>,
        program: Option<WebGlProgram>,

        coords: Option<WebGlBuffer>,
        indices: Option<WebGlBuffer>,
        texture: Option<WebGlTexture>,

        u_texture: Option<WebGlUniformLocation>,
        #[allow(dead_code)]
        a_tex_coord: Option<u32>,
    }

    impl $name {
        fn program(&self) -> Option<&WebGlProgram> {
            self.program.as_ref()
        }

        fn texture(&self) -> Option<&WebGlTexture> {
            self.texture.as_ref()
        }

        fn indices(&self) -> Option<&WebGlBuffer> {
            self.indices.as_ref()
        }

        fn coords(&self) -> Option<&WebGlBuffer> {
            self.coords.as_ref()
        }

        fn u_texture(&self) -> Option<&WebGlUniformLocation> {
            self.u_texture.as_ref()
        }

        pub fn new(
            canvas: $canvas,
            context: $context,
            color: crate::CaptureColor,
        ) -> Self {
            let vertex = initialize!(shader
                context,
                $context::VERTEX_SHADER,
                include_str!("glsl/clip.vert")
            );
            let fragment = initialize!(shader
                context,
                $context::FRAGMENT_SHADER,
                match color {
                    crate::CaptureColor::RGBL => include_str!("glsl/rgbl.frag"),
                    crate::CaptureColor::LLLA => include_str!("glsl/llla.frag"),
                    crate::CaptureColor::RGBA => include_str!("glsl/rgba.frag"),
                }
            );
            let program = vertex
                .as_ref()
                .zip(fragment.as_ref())
                .map(|(vertex, fragment)| initialize!(program context, vertex, fragment))
                .flatten();
            let texture = context.create_texture();
            let coords = context.create_buffer();
            let indices = context.create_buffer();

            let mut u_texture = None;
            let mut a_tex_coord = None;
            if let Some(program) = program.as_ref() {
                u_texture = context.get_uniform_location(&program, "u_texture");
                a_tex_coord = Some(context.get_attrib_location(&program, "a_texCoord"))
                    .filter(|v| *v != -1)
                    .map(|v| v as u32);
            }

            if let Some((coords, a_tex_coord)) = coords.as_ref().zip(a_tex_coord) {
                context.bind_buffer($context::ARRAY_BUFFER, Some(coords));
                unsafe {
                    let array = [-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0];
                    context.buffer_data_with_array_buffer_view(
                        $context::ARRAY_BUFFER,
                        &Float32Array::view(&array),
                        $context::STATIC_DRAW,
                    );
                }
                context.vertex_attrib_pointer_with_i32(
                    a_tex_coord,
                    2,
                    WebGlRenderingContext::FLOAT,
                    false,
                    0,
                    0,
                );
                context.enable_vertex_attrib_array(a_tex_coord);

                context.bind_buffer($context::ARRAY_BUFFER, None);
            }

            if let Some(indices) = indices.as_ref() {
                context.bind_buffer($context::ELEMENT_ARRAY_BUFFER, Some(indices));
                unsafe {
                    let array = [0, 1, 2, 0, 2, 3];
                    context.buffer_data_with_array_buffer_view(
                        $context::ELEMENT_ARRAY_BUFFER,
                        &Uint16Array::view(&array),
                        $context::STATIC_DRAW,
                    );
                }
                context.bind_buffer($context::ELEMENT_ARRAY_BUFFER, None);
            }

            if let Some(texture) = texture.as_ref() {
                context.bind_texture($context::TEXTURE_2D, Some(texture));
                initialize!($context texture &context);
                context.bind_texture($context::TEXTURE_2D, None);
            }

            Self {
                canvas,
                context,
                color,
                vertex,
                fragment,
                program,
                texture,
                coords,
                indices,
                u_texture,
                a_tex_coord,
            }
        }

        pub fn validate(self) -> Result<Self, Option<String>> {
            self.vertex
                .as_ref()
                .map(|vertex| validate!($context shader self.context, vertex))
                .ok_or(None)??;
            self.fragment
                .as_ref()
                .map(|fragment| validate!($context shader self.context, fragment))
                .ok_or(None)??;
            self.program
                .as_ref()
                .map(|program| validate!($context program self.context, program))
                .ok_or(None)??;

            (self.texture.is_some()
                && self.coords.is_some()
                && self.indices.is_some()
                && self.u_texture.is_some()
                && self.a_tex_coord.is_some())
            .then_some(())
            .ok_or(None)?;

            Ok(self)
        }

        pub fn from_context(
            context: $context,
            color: crate::CaptureColor,
        ) -> Option<Self> {
            context
                .canvas()?
                .dyn_into()
                .map(|canvas| Self::new(canvas, context, color))
                .ok()
        }
    }

    impl_capture_from_canvas!(
        $version,
        $name,
        $canvas,
        $context,
        $options
    );
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

            self.context.use_program(self.program());
            self.context
                .bind_buffer($context::ARRAY_BUFFER, self.coords());
            self.context
                .bind_buffer($context::ELEMENT_ARRAY_BUFFER, self.indices());
            self.context
                .bind_texture($context::TEXTURE_2D, self.texture());
            self.context.active_texture($context::TEXTURE0);
            self.context
                .pixel_storei($context::UNPACK_FLIP_Y_WEBGL, 1);

            self.context.uniform1i(self.u_texture(), 0);
            self.context.vertex_attrib_pointer_with_i32(
                    self.a_tex_coord.unwrap(),
                    2,
                    WebGlRenderingContext::FLOAT,
                    false,
                    0,
                    0,
                );
            self.context.enable_vertex_attrib_array(self.a_tex_coord.unwrap());

            match mode {
                crate::CaptureMode::Put(x, y) => {
                    if x > 0 || y > 0 {
                        self.clear();
                    } else {
                        if (sw as i32 - x) < cw as i32 || (sh as i32 - y) < ch as i32 {
                            self.clear();
                        }
                    }

                    self.context.viewport(x, y, sw as i32, sh as i32);
                }
                crate::CaptureMode::Fill => {
                    let (cw, ch) = self.capture_size();
                    self.context.viewport(0, 0, cw as i32, ch as i32);
                }
                crate::CaptureMode::Adjust => {
                    let (dw, dh) = self.capture_size();

                    if sw != dw || sh != dh {
                        self.set_capture_size(sw, sh);
                    }
                    cw = sw;
                    ch = sh;

                    self.context.viewport(0, 0, sw as i32, sh as i32);
                }
                crate::CaptureMode::Pinhole => {
                    let (cw, ch) = self.capture_size();

                    if sw < cw || sh < ch {
                        self.clear();
                    }

                    unimplemented!("CaptureMode::Pinhole");
                }
            };

            let _ = self
                .context
                .$capture_method(
                    $context::TEXTURE_2D,
                    0,
                    $context::RGBA as i32,
                    $context::RGBA,
                    $context::UNSIGNED_BYTE,
                    source,
                )
                .map(|_| {
                    self.context.draw_elements_with_i32(
                        $context::TRIANGLES,
                        6,
                        $context::UNSIGNED_SHORT,
                        0,
                    );
                    self.context.flush();
                })
                .unwrap();

            self.context.use_program(None);
            self.context
                .bind_texture($context::TEXTURE_2D, None);
            self.context
                .bind_buffer($context::ELEMENT_ARRAY_BUFFER, None);
            self.context
                .bind_buffer($context::ARRAY_BUFFER, None);

            (cw, ch)
        }

        fn retrieve(&self, buffer: &mut [u8]) {
            self.context.finish();
            self.context
                .read_pixels_with_opt_u8_array(
                    0,
                    0,
                    self.capture_width() as i32,
                    self.capture_height() as i32,
                    $context::RGBA,
                    $context::UNSIGNED_BYTE,
                    Some(buffer),
                )
                .unwrap();
        }

        fn clear(&self) {
            self.context.clear_color(0.0, 0.0, 0.0, 0.0);
            self.context.clear($context::COLOR_BUFFER_BIT);
        }
    }

    impl Drop for $name {
        fn drop(&mut self) {
            self.clear();
            let gl = &self.context;

            gl.bind_buffer($context::ARRAY_BUFFER, None);
            gl.delete_buffer(self.coords.as_ref());

            gl.bind_buffer($context::ELEMENT_ARRAY_BUFFER, None);
            gl.delete_buffer(self.indices.as_ref());

            gl.bind_texture($context::TEXTURE_2D, None);
            gl.delete_texture(self.texture.as_ref());

            gl.use_program(None);
            gl.delete_program(self.program.as_ref());

            gl.delete_shader(self.vertex.as_ref());
            gl.delete_shader(self.fragment.as_ref());
        }
    }
    };
}

#[cfg(feature = "webgl2")]
use web_sys::WebGl2RenderingContext;
#[cfg(feature = "webgl")]
use web_sys::WebGlRenderingContext;

#[cfg(feature = "html")]
pub mod html {
    use super::*;

    use web_sys::HtmlCanvasElement;

    impl_context_options!(
        HtmlContextOptionsGL
        "" experimental: bool,
        "alpha" alpha: bool,
        "depth" depth: bool,
        "stencil" stencil: bool,
        "desynchronized" desynchronized: bool,
        "antialias" antialias: bool,
        "failIfMajorPerformanceCaveat" fail_if_major_performance_caveat: bool,
        "powerPreference" power_preference: PowerPreference,
        "premultipliedAlpha" premultiplied_alpha: bool,
        "preserveDrawingBuffer" preserve_drawing_buffer: bool,
        "xrCompatible" xr_compatible: bool
    );

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum PowerPreference {
        #[default]
        Default,
        LowPower,
        HighPerformance,
    }

    impl Display for PowerPreference {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                PowerPreference::Default => write!(f, "default"),
                PowerPreference::LowPower => write!(f, "low-power"),
                PowerPreference::HighPerformance => write!(f, "high-performance"),
            }
        }
    }

    impl Into<JsValue> for PowerPreference {
        fn into(self) -> JsValue {
            JsValue::from(self.to_string())
        }
    }

    #[cfg(feature = "webgl")]
    impl_capture_gl!(
        HtmlCaptureGL
        HtmlCanvasElement,
        WebGlRenderingContext,
        HtmlContextOptionsGL,
        tex_image_2d_with_u32_and_u32_and_video,
        "webgl"
    );

    #[cfg(feature = "webgl2")]
    impl_capture_gl!(
        HtmlCaptureGL2
        HtmlCanvasElement,
        WebGl2RenderingContext,
        HtmlContextOptionsGL,
        tex_image_2d_with_u32_and_u32_and_html_video_element,
        "webgl2"
    );
}

#[cfg(feature = "offscreen")]
pub mod offscreen {
    use web_sys::OffscreenCanvas;

    use super::*;

    impl_context_options!(
        OffscreenContextOptionsGL
        "" experimental: bool,
        "alpha" alpha: bool,
        "depth" depth: bool,
        "stencil" stencil: bool,
        "antialias" antialias: bool,
        "failIfMajorPerformanceCaveat" fail_if_major_performance_caveat: bool,
        "premultipliedAlpha" premultiplied_alpha: bool,
        "preserveDrawingBuffer" preserve_drawing_buffer: bool
    );

    #[cfg(feature = "webgl")]
    impl_capture_gl!(
        OffscreenCaptureGL
        OffscreenCanvas,
        WebGlRenderingContext,
        OffscreenContextOptionsGL,
        tex_image_2d_with_u32_and_u32_and_video,
        "webgl"
    );

    #[cfg(feature = "webgl2")]
    impl_capture_gl!(
        OffscreenCaptureGL2
        OffscreenCanvas,
        WebGl2RenderingContext,
        OffscreenContextOptionsGL,
        tex_image_2d_with_u32_and_u32_and_html_video_element,
        "webgl2"
    );
}
