use anyhow::{Result, anyhow};
use make87;
use make87_messages::image::compressed::ImageJpeg;
use make87_messages::image::uncompressed::ImageRgb888;
use turbojpeg::{Compressor, Image, PixelFormat};

#[tokio::main]
async fn main() -> Result<()> {
    make87::initialize();
    
    let jpeg_quality = make87::get_config_value("JPEG_QUALITY")
        .unwrap_or_else(|| "90".to_string())
        .parse::<u8>()
        .map_err(|e| anyhow!("JPEG_QUALITY must be a valid u8: {}", e))?;

    let input_topic = "RGB_FRAME";
    let output_topic = "JPEG_FRAME";

    let subscriber = make87::resolve_topic_name(input_topic)
        .and_then(|resolved| make87::get_subscriber::<ImageRgb888>(resolved))
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

        println!("Published JPEG: {}x{}", rgb.width, rgb.height);
    }
}

fn rgb_to_jpeg(rgb: &ImageRgb888, compressor: &mut Compressor) -> Result<ImageJpeg> {
    let image = Image {
        pixels: rgb.data.as_slice(),
        width: rgb.width as usize,
        pitch: rgb.width as usize * 3,
        height: rgb.height as usize,
        format: PixelFormat::RGB,
    };

    let jpeg_data = compressor.compress_to_vec(image)?;

    Ok(ImageJpeg {
        header: rgb.header.clone(),
        data: jpeg_data,
    })
}
