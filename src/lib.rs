#[macro_use]
mod macros;
mod utils;

#[cfg(feature = "2d")]
mod _2d;

use web_sys::{js_sys, HtmlVideoElement};

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
    /// Put and crop the video frame to cover the capture area
    /// matching centers.
    Pinhole,
}

impl CaptureMode {
    pub const fn put_top_left() -> Self {
        Self::Put(0, 0)
    }
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
    fn capture(
        &self,
        source: &HtmlVideoElement,
        mode: CaptureMode,
    ) -> Result<(u32, u32), js_sys::Error>;

    /// Retrieve the grabbed frame raw data into the buffer.
    fn retrieve(&self, buffer: &mut [u8]) -> Result<(), js_sys::Error>;

    /// Get the raw data from the captured frame.
    fn data(&self) -> Result<Vec<u8>, js_sys::Error> {
        let mut buffer = vec![0; self.buffer_size()];
        self.retrieve(&mut buffer)?;
        Ok(buffer)
    }

    #[cfg(feature = "image")]
    fn image(&self) -> Result<image::DynamicImage, js_sys::Error> {
        let (width, height) = self.capture_size();
        Ok(match self.channels_count() {
            1 => image::DynamicImage::ImageLuma8(
                image::GrayImage::from_raw(width, height, self.data()?).unwrap(),
            ),
            2 => image::DynamicImage::ImageLumaA8(
                image::GrayAlphaImage::from_raw(width, height, self.data()?).unwrap(),
            ),
            3 => image::DynamicImage::ImageRgb8(
                image::RgbImage::from_raw(width, height, self.data()?).unwrap(),
            ),
            4 => image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(width, height, self.data()?).unwrap(),
            ),
            _ => panic!("Unsupported channels count"),
        })
    }

    /// Read the raw data from the video element.
    fn read(&self, source: &HtmlVideoElement, mode: CaptureMode) -> Result<Vec<u8>, js_sys::Error> {
        let (width, height) = self.capture(source, mode)?;

        let buffer_size = (width * height * self.channels_count()) as usize;
        let mut buffer = vec![0; buffer_size];

        self.retrieve(&mut buffer)?;
        Ok(buffer)
    }

    /// Clear the capture area.
    fn clear(&self);
}

#[cfg(feature = "html-2d")]
pub use _2d::html::{ColorSpaceType, HtmlCapture2D, HtmlContextOptions2D};
#[cfg(feature = "offscreen-2d")]
pub use _2d::offscreen::{OffscreenCapture2D, OffscreenContextOptions2D, OffscreenStorageType};
