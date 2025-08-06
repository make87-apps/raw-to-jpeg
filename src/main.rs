use std::error::Error;
use anyhow::{Result, anyhow};
use make87;
use make87::interfaces::zenoh::{ConfiguredSubscriber, ZenohInterface};
use make87::encodings::Encoder;
use make87_messages::image::compressed::ImageJpeg;
use make87_messages::image::uncompressed::ImageRawAny;
use turbojpeg::{Compressor, Image, PixelFormat, YuvImage, Subsamp};
use log::{info, warn, error};

macro_rules! convert_and_publish {
    ($sub:expr, $publisher:expr, $jpeg_quality:expr) => {{
        let subscriber = $sub;
        let publisher = $publisher;
        let jpeg_quality: u8 = $jpeg_quality;
        let image_raw_encoder = make87::encodings::ProtobufEncoder::<ImageRawAny>::new();
        let image_jpeg_encoder = make87::encodings::ProtobufEncoder::<ImageJpeg>::new();

        let mut compressor = Compressor::new()?;
        compressor.set_quality(jpeg_quality as i32)?;

        while let Ok(sample) = subscriber.recv_async().await {
            let message_decoded = image_raw_encoder.decode(&sample.payload().to_bytes());
            match message_decoded {
                Ok(msg) => {
                    log::info!("Received: {:?}", msg);
                    match rgb_to_jpeg(&msg, &mut compressor) {
                        Ok(jpeg) => {
                            let jpeg_encoded = image_jpeg_encoder.encode(&jpeg).unwrap();
                            publisher.put(&jpeg_encoded).await?;
                        }
                        Err(e) => log::error!("Error converting to JPEG: {e}"),
                    }
                },
                Err(e) => log::error!("Decode error: {e}"),
            }
        }
        Ok(()) as Result<(), anyhow::Error>
    }};
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();

    let application_config = make87::config::load_config_from_default_env()?;

    let jpeg_quality: u8 = match application_config.config.get("jpeg_quality") {
        Some(val) => {
            let parsed = val.to_string().parse::<u8>()
                .map_err(|_| anyhow!("jpeg_quality must be an integer between 0 and 100"))?;
            if parsed > 100 {
                return Err(anyhow!("jpeg_quality must be between 0 and 100").into());
            }
            parsed
        }
        None => {
            warn!("jpeg_quality not found in config, using default value 90");
            90
        }
    };

    let zenoh_interface = ZenohInterface::from_default_env("zenoh")?;
    let session = zenoh_interface.get_session().await?;

    let configured_subscriber = zenoh_interface.get_subscriber(&session,"raw_frame").await?;
    let publisher = zenoh_interface.get_publisher(&session, "jpeg_frame").await?;

    match configured_subscriber {
        ConfiguredSubscriber::Fifo(sub) => convert_and_publish!(&sub, &publisher, jpeg_quality)?,
        ConfiguredSubscriber::Ring(sub) => convert_and_publish!(&sub, &publisher, jpeg_quality)?,
    }

    Ok(())
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