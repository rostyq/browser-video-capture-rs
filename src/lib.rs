#[macro_use]
mod macros;
mod utils;

#[cfg(feature = "2d")]
mod d2;
#[cfg(feature = "gl")]
mod gl;

use web_sys::{js_sys, HtmlVideoElement};

macro_rules! impl_enum_from {
    ($from:ty => $typ:ty:$name:tt) => {
        impl From<$from> for $typ {
            fn from(value: $from) -> Self {
                Self::$name(value)
            }
        }
    };
}

macro_rules! enum_method {
    ($name:tt ($( $arg:tt: $typ:ty ),*) => $ret:ty) => {
        fn $name(&self, $($arg: $typ),*) -> $ret {
            match self {
                #[cfg(feature = "html-2d")]
                Self::Html2D(c) => c.$name($($arg),*),
                #[cfg(feature = "offscreen-2d")]
                Self::Offscreen2D(c) => c.$name($($arg),*),
                #[cfg(all(feature = "html", feature = "webgl"))]
                Self::HtmlGL(c) => c.$name($($arg),*),
                #[cfg(all(feature = "html", feature = "webgl2"))]
                Self::HtmlGL2(c) => c.$name($($arg),*),
                #[cfg(all(feature = "offscreen", feature = "webgl"))]
                Self::OffscreenGL(c) => c.$name($($arg),*),
                #[cfg(all(feature = "offscreen", feature = "webgl2"))]
                Self::OffscreenGL2(c) => c.$name($($arg),*),
            }
        }
    };
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CaptureMode {
    /// Put the video frame at `(x, y)` on the capture area
    /// with original size ignoring capture size.
    Put(i32, i32),
    /// Fill the capture area with the entire video frame.
    /// Same as `object-fit: fill` CSS property.
    Fill,
    /// Resize the capture area to fit the entire video frame.
    /// This is the default mode.
    #[default]
    Adjust,
    /// Put and scale the video frame to cover the capture area
    /// matching centers.
    Pinhole,
}

impl CaptureMode {
    pub const fn put_top_left() -> Self {
        Self::Put(0, 0)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureColor {
    /// Output data as RGBA.
    #[default]
    RGBA,
    /// Output data as RGBA but Alpha channel is luminosity.
    RGBL,
    /// Output data as grayscale RGBA.
    LLLA,
}

pub trait CaptureArea {
    /// Get the width of the available capture area in pixels.
    fn capture_width(&self) -> u32;

    /// Get the height of the available capture area in pixels.
    fn capture_height(&self) -> u32;

    /// Get the size of the available capture area in pixels.
    fn capture_size(&self) -> (u32, u32) {
        (self.capture_width(), self.capture_height())
    }

    /// Get the capture area in pixels.
    fn capture_area(&self) -> u32 {
        self.capture_width() * self.capture_height()
    }

    /// Set the width for the capture area.
    fn set_capture_width(&self, width: u32);

    /// Set the height for the capture area.
    fn set_capture_height(&self, height: u32);

    /// Set the size for the capture area.
    fn set_capture_size(&self, width: u32, height: u32) {
        self.set_capture_width(width);
        self.set_capture_height(height);
    }
}

pub trait BrowserVideoCapture: CaptureArea {
    /// Get the number of channels in the capture buffer.
    fn channels_count(&self) -> u32 {
        4
    }

    #[cfg(feature = "image")]
    fn color_type(&self) -> image::ColorType {
        match self.channels_count() {
            1 => image::ColorType::L8,
            2 => image::ColorType::La8,
            3 => image::ColorType::Rgb8,
            4 => image::ColorType::Rgba8,
            _ => panic!("Unsupported channels count"),
        }
    }

    /// Get the size of the capture buffer in bytes.
    fn buffer_size(&self) -> usize {
        (self.capture_area() * self.channels_count()) as usize
    }

    /// Capture a frame from the video element.
    fn capture(&self, source: &HtmlVideoElement, mode: CaptureMode) -> (u32, u32);

    /// Retrieve the grabbed frame raw data into the buffer.
    fn retrieve(&self, buffer: &mut [u8]);

    /// Get the raw data from the captured frame.
    fn data(&self) -> Vec<u8> {
        let mut buffer = vec![0; self.buffer_size()];
        self.retrieve(&mut buffer);
        buffer
    }

    #[cfg(feature = "image")]
    fn image(&self) -> Option<image::DynamicImage> {
        let (width, height) = self.capture_size();
        Some(match self.channels_count() {
            1 => image::DynamicImage::ImageLuma8(image::GrayImage::from_raw(
                width,
                height,
                self.data(),
            )?),
            2 => image::DynamicImage::ImageLumaA8(image::GrayAlphaImage::from_raw(
                width,
                height,
                self.data(),
            )?),
            3 => image::DynamicImage::ImageRgb8(image::RgbImage::from_raw(
                width,
                height,
                self.data(),
            )?),
            4 => image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(
                width,
                height,
                self.data(),
            )?),
            _ => panic!("Unsupported channels count"),
        })
    }

    /// Read the raw data from the video element.
    fn read(&self, source: &HtmlVideoElement, mode: CaptureMode) -> Vec<u8> {
        let (width, height) = self.capture(source, mode);

        let buffer_size = (width * height * self.channels_count()) as usize;

        if buffer_size > 0 {
            let mut buffer = vec![0; buffer_size];
            self.retrieve(&mut buffer);
            buffer
        } else {
            return Vec::new();
        }
    }

    /// Clear the capture area.
    fn clear(&self);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportedCanvas {
    #[cfg(feature = "html")]
    Html(web_sys::HtmlCanvasElement),
    #[cfg(feature = "offscreen")]
    Offscreen(web_sys::OffscreenCanvas),
}

#[cfg(feature = "html")]
impl_enum_from!(web_sys::HtmlCanvasElement => SupportedCanvas:Html);
#[cfg(feature = "offscreen")]
impl_enum_from!(web_sys::OffscreenCanvas => SupportedCanvas:Offscreen);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportedContext {
    #[cfg(feature = "html-2d")]
    Html2D(web_sys::CanvasRenderingContext2d),
    #[cfg(feature = "offscreen-2d")]
    Ofscreen2D(web_sys::OffscreenCanvasRenderingContext2d),
    #[cfg(feature = "webgl")]
    WebGL(web_sys::WebGlRenderingContext),
    #[cfg(feature = "webgl2")]
    WebGL2(web_sys::WebGl2RenderingContext),
}

#[cfg(feature = "html-2d")]
impl_enum_from!(web_sys::CanvasRenderingContext2d => SupportedContext:Html2D);
#[cfg(feature = "offscreen-2d")]
impl_enum_from!(web_sys::OffscreenCanvasRenderingContext2d => SupportedContext:Ofscreen2D);
#[cfg(feature = "webgl")]
impl_enum_from!(web_sys::WebGlRenderingContext => SupportedContext:WebGL);
#[cfg(feature = "webgl2")]
impl_enum_from!(web_sys::WebGl2RenderingContext => SupportedContext:WebGL2);

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct BrowserCaptureBuilder {
    pub context: Option<SupportedContext>,
    pub canvas: Option<SupportedCanvas>,
    pub color: Option<CaptureColor>,
    pub options: Option<SupportedOptions>,
}

impl BrowserCaptureBuilder {
    pub fn color(mut self, color: CaptureColor) -> Self {
        self.color = Some(color);
        self
    }

    pub fn canvas(mut self, canvas: SupportedCanvas) -> Self {
        self.canvas = Some(canvas);
        self
    }

    pub fn context(mut self, context: SupportedContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn options(mut self, options: SupportedOptions) -> Self {
        self.options = Some(options);
        self
    }

    #[cfg(feature = "html")]
    pub fn html_canvas(self, canvas: web_sys::HtmlCanvasElement) -> Self {
        self.canvas(SupportedCanvas::Html(canvas))
    }

    #[cfg(feature = "offscreen")]
    pub fn offscreen_canvas(self, canvas: web_sys::OffscreenCanvas) -> Self {
        self.canvas(SupportedCanvas::Offscreen(canvas))
    }

    #[cfg(feature = "html-2d")]
    pub fn html_2d(self, context: web_sys::CanvasRenderingContext2d) -> Self {
        self.context(SupportedContext::Html2D(context))
    }

    #[cfg(feature = "offscreen-2d")]
    pub fn offscreen_2d(self, context: web_sys::OffscreenCanvasRenderingContext2d) -> Self {
        self.context(SupportedContext::Ofscreen2D(context))
    }

    #[cfg(feature = "webgl")]
    pub fn webgl(self, context: web_sys::WebGlRenderingContext) -> Self {
        self.context(SupportedContext::WebGL(context))
    }

    #[cfg(feature = "webgl2")]
    pub fn webgl2(self, context: web_sys::WebGl2RenderingContext) -> Self {
        self.context(SupportedContext::WebGL2(context))
    }

    pub fn build(self) -> Option<Result<BrowserCapture, js_sys::Error>> {
        match (self.canvas, self.context, self.options) {
            #[cfg(feature = "html-2d")]
            (Some(SupportedCanvas::Html(canvas)), Some(SupportedContext::Html2D(context)), _) => {
                Some(Ok(HtmlCapture2D::new(
                    canvas,
                    context,
                    self.color.unwrap_or_default(),
                )
                .into()))
            }
            #[cfg(feature = "html-2d")]
            (
                Some(SupportedCanvas::Html(canvas)),
                None,
                Some(SupportedOptions::Html2D(options)),
            ) => Some(
                HtmlCapture2D::from_canvas_with_options(
                    canvas,
                    self.color.unwrap_or_default(),
                    options,
                )
                .transpose()?
                .map(Into::into),
            ),
            #[cfg(feature = "offscreen-2d")]
            (
                Some(SupportedCanvas::Offscreen(canvas)),
                Some(SupportedContext::Ofscreen2D(context)),
                _,
            ) => Some(Ok(OffscreenCapture2D::new(
                canvas,
                context,
                self.color.unwrap_or_default(),
            )
            .into())),
            #[cfg(feature = "offscreen-2d")]
            (
                Some(SupportedCanvas::Offscreen(canvas)),
                None,
                Some(SupportedOptions::Offscreen2D(options)),
            ) => Some(
                OffscreenCapture2D::from_canvas_with_options(
                    canvas,
                    self.color.unwrap_or_default(),
                    options,
                )
                .transpose()?
                .map(Into::into),
            ),
            #[cfg(all(feature = "html", feature = "webgl"))]
            (Some(SupportedCanvas::Html(canvas)), Some(SupportedContext::WebGL(context)), _) => {
                Some(Ok(HtmlCaptureGL::new(
                    canvas,
                    context,
                    self.color.unwrap_or_default(),
                )
                .into()))
            }
            #[cfg(all(feature = "html", feature = "webgl2"))]
            (Some(SupportedCanvas::Html(canvas)), Some(SupportedContext::WebGL2(context)), _) => {
                Some(Ok(HtmlCaptureGL2::new(
                    canvas,
                    context,
                    self.color.unwrap_or_default(),
                )
                .into()))
            }
            #[cfg(all(feature = "html", feature = "webgl"))]
            (
                Some(SupportedCanvas::Html(canvas)),
                None,
                Some(SupportedOptions::HtmlGL(options)),
            ) if matches!(options.version, GLVersion::WebGL) => Some(
                HtmlCaptureGL::from_canvas_with_options(
                    canvas,
                    self.color.unwrap_or_default(),
                    options,
                )
                .transpose()?
                .map(|c| c.validate().ok())
                .transpose()?
                .map(Into::into),
            ),
            #[cfg(all(feature = "html", feature = "webgl2"))]
            (
                Some(SupportedCanvas::Html(canvas)),
                None,
                Some(SupportedOptions::HtmlGL(options)),
            ) if matches!(options.version, GLVersion::WebGL2) => Some(
                HtmlCaptureGL2::from_canvas_with_options(
                    canvas,
                    self.color.unwrap_or_default(),
                    options,
                )
                .transpose()?
                .map(|c| c.validate().ok())
                .transpose()?
                .map(Into::into),
            ),
            #[cfg(all(feature = "offscreen", feature = "webgl"))]
            (
                Some(SupportedCanvas::Offscreen(canvas)),
                Some(SupportedContext::WebGL(context)),
                _,
            ) => Some(Ok(OffscreenCaptureGL::new(
                canvas,
                context,
                self.color.unwrap_or_default(),
            )
            .into())),
            #[cfg(all(feature = "offscreen", feature = "webgl2"))]
            (
                Some(SupportedCanvas::Offscreen(canvas)),
                Some(SupportedContext::WebGL2(context)),
                _,
            ) => Some(Ok(OffscreenCaptureGL2::new(
                canvas,
                context,
                self.color.unwrap_or_default(),
            )
            .into())),
            #[cfg(all(feature = "offscreen", feature = "webgl"))]
            (
                Some(SupportedCanvas::Offscreen(canvas)),
                None,
                Some(SupportedOptions::OffscreenGL(options)),
            ) if matches!(options.version, GLVersion::WebGL) => Some(
                OffscreenCaptureGL::from_canvas_with_options(
                    canvas,
                    self.color.unwrap_or_default(),
                    options,
                )
                .transpose()?
                .map(|c| c.validate().ok())
                .transpose()?
                .map(Into::into),
            ),
            #[cfg(all(feature = "offscreen", feature = "webgl"))]
            (
                Some(SupportedCanvas::Offscreen(canvas)),
                None,
                Some(SupportedOptions::OffscreenGL(options)),
            ) if matches!(options.version, GLVersion::WebGL2) => Some(
                OffscreenCaptureGL2::from_canvas_with_options(
                    canvas,
                    self.color.unwrap_or_default(),
                    options,
                )
                .transpose()?
                .map(|c| c.validate().ok())
                .transpose()?
                .map(Into::into),
            ),
            _ => None,
        }
    }
}

#[cfg(all(feature = "html", feature = "2d"))]
pub use d2::html::ColorSpaceType;
#[cfg(feature = "html-2d")]
pub use d2::html::{HtmlCapture2D, HtmlContextOptions2D};
#[cfg(all(feature = "offscreen", feature = "2d"))]
pub use d2::offscreen::OffscreenStorageType;
#[cfg(feature = "offscreen-2d")]
pub use d2::offscreen::{OffscreenCapture2D, OffscreenContextOptions2D};

#[cfg(feature = "gl")]
pub use gl::GLVersion;
#[cfg(all(feature = "html", feature = "gl"))]
pub use gl::html::{PowerPreference, HtmlContextOptionsGL};
#[cfg(all(feature = "offscreen", feature = "gl"))]
pub use gl::offscreen::OffscreenContextOptionsGL;

#[cfg(all(feature = "html", feature = "webgl"))]
pub use gl::html::HtmlCaptureGL;
#[cfg(all(feature = "html", feature = "webgl2"))]
pub use gl::html::HtmlCaptureGL2;
#[cfg(all(feature = "offscreen", feature = "webgl"))]
pub use gl::offscreen::OffscreenCaptureGL;
#[cfg(all(feature = "offscreen", feature = "webgl2"))]
pub use gl::offscreen::OffscreenCaptureGL2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserCapture {
    #[cfg(feature = "html-2d")]
    Html2D(HtmlCapture2D),
    #[cfg(feature = "offscreen-2d")]
    Offscreen2D(OffscreenCapture2D),
    #[cfg(all(feature = "html", feature = "webgl"))]
    HtmlGL(HtmlCaptureGL),
    #[cfg(all(feature = "html", feature = "webgl2"))]
    HtmlGL2(HtmlCaptureGL2),
    #[cfg(all(feature = "offscreen", feature = "webgl"))]
    OffscreenGL(OffscreenCaptureGL),
    #[cfg(all(feature = "offscreen", feature = "webgl2"))]
    OffscreenGL2(OffscreenCaptureGL2),
}

#[cfg(feature = "html-2d")]
impl_enum_from!(HtmlCapture2D => BrowserCapture:Html2D);
#[cfg(feature = "offscreen-2d")]
impl_enum_from!(OffscreenCapture2D => BrowserCapture:Offscreen2D);
#[cfg(all(feature = "html", feature = "webgl"))]
impl_enum_from!(HtmlCaptureGL => BrowserCapture:HtmlGL);
#[cfg(all(feature = "offscreen", feature = "webgl"))]
impl_enum_from!(OffscreenCaptureGL => BrowserCapture:OffscreenGL);
#[cfg(all(feature = "html", feature = "webgl2"))]
impl_enum_from!(HtmlCaptureGL2 => BrowserCapture:HtmlGL2);
#[cfg(all(feature = "offscreen", feature = "webgl2"))]
impl_enum_from!(OffscreenCaptureGL2 => BrowserCapture:OffscreenGL2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedOptions {
    #[cfg(feature = "html-2d")]
    Html2D(HtmlContextOptions2D),
    #[cfg(feature = "offscreen-2d")]
    Offscreen2D(OffscreenContextOptions2D),
    #[cfg(all(feature = "html", feature = "gl"))]
    HtmlGL(HtmlContextOptionsGL),
    #[cfg(all(feature = "offscreen", feature = "gl"))]
    OffscreenGL(OffscreenContextOptionsGL),
}

#[cfg(feature = "html-2d")]
impl_enum_from!(HtmlContextOptions2D => SupportedOptions:Html2D);
#[cfg(feature = "offscreen-2d")]
impl_enum_from!(OffscreenContextOptions2D => SupportedOptions:Offscreen2D);
#[cfg(all(feature = "html", feature = "gl"))]
impl_enum_from!(HtmlContextOptionsGL => SupportedOptions:HtmlGL);
#[cfg(all(feature = "offscreen", feature = "gl"))]
impl_enum_from!(OffscreenContextOptionsGL => SupportedOptions:OffscreenGL);

impl From<BrowserCapture> for Box<dyn BrowserVideoCapture> {
    fn from(value: BrowserCapture) -> Self {
        match value {
            #[cfg(feature = "html-2d")]
            BrowserCapture::Html2D(c) => Box::new(c),
            #[cfg(feature = "offscreen-2d")]
            BrowserCapture::Offscreen2D(c) => Box::new(c),
            #[cfg(all(feature = "html", feature = "webgl"))]
            BrowserCapture::HtmlGL(c) => Box::new(c),
            #[cfg(all(feature = "html", feature = "webgl2"))]
            BrowserCapture::HtmlGL2(c) => Box::new(c),
            #[cfg(all(feature = "offscreen", feature = "webgl"))]
            BrowserCapture::OffscreenGL(c) => Box::new(c),
            #[cfg(all(feature = "offscreen", feature = "webgl2"))]
            BrowserCapture::OffscreenGL2(c) => Box::new(c),
        }
    }
}

impl CaptureArea for BrowserCapture {
    enum_method!(capture_width () => u32);
    enum_method!(capture_height () => u32);
    enum_method!(set_capture_width (width: u32) => ());
    enum_method!(set_capture_height (height: u32) => ());
}

impl BrowserVideoCapture for BrowserCapture {
    enum_method!(channels_count () => u32);
    enum_method!(buffer_size () => usize);
    enum_method!(capture (source: &HtmlVideoElement, mode: CaptureMode) => (u32, u32));
    enum_method!(retrieve (buffer: &mut [u8]) => ());
    enum_method!(data () => Vec<u8>);
    #[cfg(feature = "image")]
    enum_method!(image () => Option<image::DynamicImage>);
    enum_method!(read (source: &HtmlVideoElement, mode: CaptureMode) => Vec<u8>);
    enum_method!(clear () => ());
}
