use anyhow::{Result, anyhow};
use make87_messages::image::compressed::ImageJpeg;
use make87_messages::image::uncompressed::ImageRawAny;
use turbojpeg::{Compressor, Image, PixelFormat, YuvImage, Subsamp};

pub fn rgb_to_jpeg(rgb_any: &ImageRawAny, compressor: &mut Compressor) -> Result<ImageJpeg> {
    use make87_messages::image::uncompressed::image_raw_any::Image as RawImageVariant;

    match &rgb_any.image {
        Some(RawImageVariant::Rgb888(rgb888)) => {
            let pixels = rgb888.data.as_slice();
            let width = rgb888.width as usize;
            let height = rgb888.height as usize;
            let pitch = width * 3;
            let image = Image {
                pixels,
                width,
                pitch,
                height,
                format: PixelFormat::RGB,
            };
            let jpeg_data = compressor.compress_to_vec(image)?;
            Ok(ImageJpeg {
                header: rgb_any.header.clone(),
                data: jpeg_data,
            })
        }
        Some(RawImageVariant::Rgba8888(rgba8888)) => {
            let pixels = rgba8888.data.as_slice();
            let width = rgba8888.width as usize;
            let height = rgba8888.height as usize;
            let pitch = width * 4;
            let image = Image {
                pixels,
                width,
                pitch,
                height,
                format: PixelFormat::RGBA,
            };
            let jpeg_data = compressor.compress_to_vec(image)?;
            Ok(ImageJpeg {
                header: rgb_any.header.clone(),
                data: jpeg_data,
            })
        }
        Some(RawImageVariant::Yuv420(yuv420)) => {
            let width = yuv420.width as usize;
            let height = yuv420.height as usize;
            let yuv_data = yuv420.data.as_slice();
            let yuv_image = YuvImage {
                pixels: yuv_data,
                width,
                align: 1,
                height,
                subsamp: Subsamp::Sub2x2, // YUV420
            };
            let jpeg_data = compressor.compress_yuv_to_vec(yuv_image)?;
            Ok(ImageJpeg {
                header: rgb_any.header.clone(),
                data: jpeg_data,
            })
        }
        Some(RawImageVariant::Yuv422(yuv422)) => {
            let width = yuv422.width as usize;
            let height = yuv422.height as usize;
            let yuv_data = yuv422.data.as_slice();
            let yuv_image = YuvImage {
                pixels: yuv_data,
                width,
                align: 1,
                height,
                subsamp: Subsamp::Sub2x1, // YUV422
            };
            let jpeg_data = compressor.compress_yuv_to_vec(yuv_image)?;
            Ok(ImageJpeg {
                header: rgb_any.header.clone(),
                data: jpeg_data,
            })
        }
        Some(RawImageVariant::Yuv444(yuv444)) => {
            let width = yuv444.width as usize;
            let height = yuv444.height as usize;
            let yuv_data = yuv444.data.as_slice();
            let yuv_image = YuvImage {
                pixels: yuv_data,
                width,
                align: 1,
                height,
                subsamp: Subsamp::None, // YUV444
            };
            let jpeg_data = compressor.compress_yuv_to_vec(yuv_image)?;
            Ok(ImageJpeg {
                header: rgb_any.header.clone(),
                data: jpeg_data,
            })
        }
        Some(RawImageVariant::Nv12(nv12)) => {
            let width = nv12.width as usize;
            let height = nv12.height as usize;
            let nv12_data = nv12.data.as_slice();

            // NV12 format: Y plane followed by interleaved UV plane
            let y_size = width * height;
            let uv_size = y_size / 2; // UV plane is half the size (2x2 subsampling)

            if nv12_data.len() < y_size + uv_size {
                return Err(anyhow!("NV12 data too small: expected {}, got {}", y_size + uv_size, nv12_data.len()));
            }

            // Create planar YUV420 data
            let mut yuv420_data = Vec::with_capacity(y_size + uv_size);

            // Copy Y plane as-is
            yuv420_data.extend_from_slice(&nv12_data[0..y_size]);

            // Convert interleaved UV to separate U and V planes
            let uv_plane = &nv12_data[y_size..y_size + uv_size];

            // Extract U components (even indices in UV plane)
            for i in (0..uv_size).step_by(2) {
                yuv420_data.push(uv_plane[i]);
            }

            // Extract V components (odd indices in UV plane)
            for i in (1..uv_size).step_by(2) {
                yuv420_data.push(uv_plane[i]);
            }

            let yuv_image = YuvImage {
                pixels: yuv420_data.as_slice(),
                width,
                align: 1,
                height,
                subsamp: Subsamp::Sub2x2, // YUV420 (converted from NV12)
            };
            let jpeg_data = compressor.compress_yuv_to_vec(yuv_image)?;
            Ok(ImageJpeg {
                header: rgb_any.header.clone(),
                data: jpeg_data,
            })
        }
        None => Err(anyhow!("No image data in ImageRawAny")),
    }
}

