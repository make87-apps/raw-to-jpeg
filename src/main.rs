use std::error::Error;
use anyhow::{Result, anyhow};
use make87;
use make87::interfaces::zenoh::{ConfiguredSubscriber, ZenohInterface};
use make87::encodings::Encoder;
use make87_messages::image::compressed::ImageJpeg;
use make87_messages::image::uncompressed::ImageRawAny;
use turbojpeg::Compressor;
use log::{info, warn, error};
use raw_to_jpeg::rgb_to_jpeg;

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
                    log::info!("Received image frame");
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
