use anyhow::{Result, anyhow};
use make87;
use make87_messages::image::compressed::ImageJpeg;
use make87_messages::image::uncompressed::ImageRawAny;
use turbojpeg::{Compressor, Image, PixelFormat, YuvImage, Subsamp};

#[tokio::main]
async fn main() -> Result<()> {
    make87::initialize();
    
    let jpeg_quality = make87::get_config_value("JPEG_QUALITY")
        .unwrap_or_else(|| "90".to_string())
        .parse::<u8>()
        .map_err(|e| anyhow!("JPEG_QUALITY must be a valid u8: {}", e))?;

    let input_topic = "RAW_FRAME";
    let output_topic = "JPEG_FRAME";

    let subscriber = make87::resolve_topic_name(input_topic)
        .and_then(|resolved| make87::get_subscriber::<ImageRawAny>(resolved))
        .expect("Failed to resolve or subscribe");

    let publisher = make87::resolve_topic_name(output_topic)
        .and_then(|resolved| make87::get_publisher::<ImageJpeg>(resolved))
        .expect("Failed to resolve or create publisher");

    let mut compressor = Compressor::new()?;
    compressor.set_quality(jpeg_quality as i32)?;

    loop {
        let rgb = subscriber.receive_async().await.map_err(|e| anyhow!(e))?;

        let jpeg = rgb_to_jpeg(&rgb, &mut compressor)?;

        publisher.publish_async(&jpeg)
            .await
            .map_err(|e| anyhow!(e))?;
    }
}

fn rgb_to_jpeg(rgb_any: &ImageRawAny, compressor: &mut Compressor) -> Result<ImageJpeg> {
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
        None => Err(anyhow!("No image data in ImageRawAny")),
    }
}
