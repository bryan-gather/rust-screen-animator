use image::{DynamicImage, ImageBuffer, Rgba};

///
/// No-op capturer
///
/// A capturer that does nothign, for testing
///
use crate::capturer::Capturer;

pub(crate) struct NoopCapturer {
    // ...
}

impl Capturer for NoopCapturer {
    // ...

    fn init(&self) {}

    fn capture_window(&self, window_id: u64) -> Result<DynamicImage, ()> {
        // Define the dimensions of the new image
        let width = 800;
        let height = 600;

        // Create a new ImageBuffer with the specified dimensions and fill it with a color
        let mut img_buf = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 255, 255]));

        // Optionally, you can draw or manipulate the image buffer here
        // For example, setting a specific pixel:
        img_buf.put_pixel(400, 300, Rgba([255, 0, 0, 255]));

        // Convert the ImageBuffer to a DynamicImage
        let dynamic_image = DynamicImage::ImageRgba8(img_buf);
        Ok(dynamic_image)
    }

    fn list_windows(&self) -> Result<Vec<(u64, std::ffi::OsString)>, ()> {
        Ok(vec![])
    }
}
