use anyhow::Result;
use make87_messages::core::Header;
use make87_messages::google::protobuf::Timestamp;
use make87_messages::image::uncompressed::image_raw_any::Image as RawImageVariant;
use make87_messages::image::uncompressed::{ImageNv12, ImageRawAny, ImageRgb888, ImageYuv420, ImageYuv422, ImageYuv444};
use raw_to_jpeg::rgb_to_jpeg;
use std::fs;
use std::path::Path;
use turbojpeg::Compressor;

/// Test data directory structure:
/// tests/data/
/// ├── input/
/// │   ├── test_frame_640x480.yuv420
/// │   ├── test_frame_640x480.yuv422
/// │   ├── test_frame_640x480.yuv444
/// │   ├── test_frame_640x480.nv12
/// │   ├── test_frame_640x480.rgb888
/// │   └── test_frame_640x480.rgba8888
/// └── expected/
///     ├── test_frame_640x480_yuv420.jpg
///     ├── test_frame_640x480_yuv422.jpg
///     ├── test_frame_640x480_yuv444.jpg
///     ├── test_frame_640x480_nv12.jpg
///     ├── test_frame_640x480_rgb888.jpg
///     └── test_frame_640x480_rgba8888.jpg

const TEST_WIDTH: u32 = 176;
const TEST_HEIGHT: u32 = 144;
const JPEG_QUALITY: i32 = 90;

fn create_test_header() -> Header {
    Header {
        timestamp: Some(Timestamp {
            seconds: 1234567890,
            nanos: 0,
        }),
        ..Default::default()
    }
}

fn load_test_file(filename: &str) -> Result<Vec<u8>> {
    let path = Path::new("tests/data/input").join(filename);
    Ok(fs::read(path)?)
}

fn save_output_jpeg(data: &[u8], filename: &str) -> Result<()> {
    let output_dir = Path::new("tests/data/output");
    fs::create_dir_all(output_dir)?;
    let path = output_dir.join(filename);
    fs::write(path, data)?;
    Ok(())
}

#[test]
fn test_rgb888_conversion() -> Result<()> {
    let raw_data = load_test_file("tulips_rgb444_prog_packed_qcif.yuv")?;

    let header = create_test_header();

    let rgb888 = ImageRgb888 {
        header: Some(header.clone()),
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        data: raw_data,
    };

    let image_raw = ImageRawAny {
        header: Some(header),
        image: Some(RawImageVariant::Rgb888(rgb888)),
    };

    let mut compressor = Compressor::new()?;
    compressor.set_quality(JPEG_QUALITY)?;

    let jpeg_result = rgb_to_jpeg(&image_raw, &mut compressor)?;

    // Verify JPEG header is present
    assert!(jpeg_result.header.is_some());

    // Verify JPEG data starts with JPEG magic bytes (0xFF 0xD8)
    assert!(jpeg_result.data.len() > 2);
    assert_eq!(jpeg_result.data[0], 0xFF);
    assert_eq!(jpeg_result.data[1], 0xD8);

    // Save for visual inspection
    save_output_jpeg(&jpeg_result.data, "test_frame_640x480_rgb888_output.jpg")?;

    println!("RGB888 conversion successful. Output saved to tests/data/output/test_frame_640x480_rgb888_output.jpg");
    Ok(())
}

#[test]
fn test_yuv420_conversion() -> Result<()> {
    let raw_data = load_test_file("tulips_yuv420_prog_planar_qcif.yuv")?;

    let header = create_test_header();

    let yuv420 = ImageYuv420 {
        header: Some(header.clone()),
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        data: raw_data,
    };

    let image_raw = ImageRawAny {
        header: Some(header),
        image: Some(RawImageVariant::Yuv420(yuv420)),
    };

    let mut compressor = Compressor::new()?;
    compressor.set_quality(JPEG_QUALITY)?;

    let jpeg_result = rgb_to_jpeg(&image_raw, &mut compressor)?;

    // Verify JPEG data
    assert!(jpeg_result.data.len() > 2);
    assert_eq!(jpeg_result.data[0], 0xFF);
    assert_eq!(jpeg_result.data[1], 0xD8);

    save_output_jpeg(&jpeg_result.data, "test_frame_640x480_yuv420_output.jpg")?;

    println!("YUV420 conversion successful. Output saved to tests/data/output/test_frame_640x480_yuv420_output.jpg");
    Ok(())
}

#[test]
fn test_yuv422_conversion() -> Result<()> {
    let raw_data = load_test_file("tulips_yuv422_prog_planar_qcif.yuv")?;

    let header = create_test_header();

    let yuv422 = ImageYuv422 {
        header: Some(header.clone()),
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        data: raw_data,
    };

    let image_raw = ImageRawAny {
        header: Some(header),
        image: Some(RawImageVariant::Yuv422(yuv422)),
    };

    let mut compressor = Compressor::new()?;
    compressor.set_quality(JPEG_QUALITY)?;

    let jpeg_result = rgb_to_jpeg(&image_raw, &mut compressor)?;

    // Verify JPEG data
    assert!(jpeg_result.data.len() > 2);
    assert_eq!(jpeg_result.data[0], 0xFF);
    assert_eq!(jpeg_result.data[1], 0xD8);

    save_output_jpeg(&jpeg_result.data, "test_frame_640x480_yuv422_output.jpg")?;

    println!("YUV422 conversion successful. Output saved to tests/data/output/test_frame_640x480_yuv422_output.jpg");
    Ok(())
}

#[test]
fn test_yuv444_conversion() -> Result<()> {
    let raw_data = load_test_file("tulips_yuv444_prog_planar_qcif.yuv")?;

    let header = create_test_header();

    let yuv444 = ImageYuv444 {
        header: Some(header.clone()),
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        data: raw_data,
    };

    let image_raw = ImageRawAny {
        header: Some(header),
        image: Some(RawImageVariant::Yuv444(yuv444)),
    };

    let mut compressor = Compressor::new()?;
    compressor.set_quality(JPEG_QUALITY)?;

    let jpeg_result = rgb_to_jpeg(&image_raw, &mut compressor)?;

    // Verify JPEG data
    assert!(jpeg_result.data.len() > 2);
    assert_eq!(jpeg_result.data[0], 0xFF);
    assert_eq!(jpeg_result.data[1], 0xD8);

    save_output_jpeg(&jpeg_result.data, "test_frame_640x480_yuv444_output.jpg")?;

    println!("YUV444 conversion successful. Output saved to tests/data/output/test_frame_640x480_yuv444_output.jpg");
    Ok(())
}

#[test]
fn test_nv12_conversion() -> Result<()> {
    let raw_data = load_test_file("tulips_nv12_prog_qcif.yuv")?;

    let header = create_test_header();

    let nv12 = ImageNv12 {
        header: Some(header.clone()),
        width: TEST_WIDTH,
        height: TEST_HEIGHT,
        data: raw_data,
    };

    let image_raw = ImageRawAny {
        header: Some(header),
        image: Some(RawImageVariant::Nv12(nv12)),
    };

    let mut compressor = Compressor::new()?;
    compressor.set_quality(JPEG_QUALITY)?;

    let jpeg_result = rgb_to_jpeg(&image_raw, &mut compressor)?;

    // Verify JPEG data
    assert!(jpeg_result.data.len() > 2);
    assert_eq!(jpeg_result.data[0], 0xFF);
    assert_eq!(jpeg_result.data[1], 0xD8);

    save_output_jpeg(&jpeg_result.data, "test_frame_640x480_nv12_output.jpg")?;

    println!("NV12 conversion successful. Output saved to tests/data/output/test_frame_640x480_nv12_output.jpg");
    Ok(())
}


#[cfg(test)]
mod benchmark_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore] // Run with `cargo test benchmark_tests -- --ignored`
    fn benchmark_conversion_performance() -> Result<()> {
        const NUM_RUNS: usize = 100;

        // This test requires the input files to exist
        let formats = [
            ("rgb888", "tulips_rgb444_prog_packed_qcif.yuv"),
            ("yuv420", "tulips_yuv420_prog_planar_qcif.yuv"),
            ("yuv422", "tulips_yuv422_prog_planar_qcif.yuv"),
            ("yuv444", "tulips_yuv444_prog_planar_qcif.yuv"),
            ("nv12", "tulips_nv12_prog_qcif.yuv"),
        ];

        println!("Running {} iterations per format...\n", NUM_RUNS);

        for (format_name, filename) in formats.iter() {
            if let Ok(raw_data) = load_test_file(filename) {
                let header = create_test_header();

                // Create appropriate image variant based on format
                let image_raw = match *format_name {
                    "rgb888" => ImageRawAny {
                        header: Some(header.clone()),
                        image: Some(RawImageVariant::Rgb888(ImageRgb888 {
                            header: Some(header.clone()),
                            width: TEST_WIDTH,
                            height: TEST_HEIGHT,
                            data: raw_data,
                        })),
                    },
                    "yuv420" => ImageRawAny {
                        header: Some(header.clone()),
                        image: Some(RawImageVariant::Yuv420(ImageYuv420 {
                            header: Some(header.clone()),
                            width: TEST_WIDTH,
                            height: TEST_HEIGHT,
                            data: raw_data,
                        })),
                    },
                    "yuv422" => ImageRawAny {
                        header: Some(header.clone()),
                        image: Some(RawImageVariant::Yuv422(ImageYuv422 {
                            header: Some(header.clone()),
                            width: TEST_WIDTH,
                            height: TEST_HEIGHT,
                            data: raw_data,
                        })),
                    },
                    "yuv444" => ImageRawAny {
                        header: Some(header.clone()),
                        image: Some(RawImageVariant::Yuv444(ImageYuv444 {
                            header: Some(header.clone()),
                            width: TEST_WIDTH,
                            height: TEST_HEIGHT,
                            data: raw_data,
                        })),
                    },
                    "nv12" => ImageRawAny {
                        header: Some(header.clone()),
                        image: Some(RawImageVariant::Nv12(ImageNv12 {
                            header: Some(header.clone()),
                            width: TEST_WIDTH,
                            height: TEST_HEIGHT,
                            data: raw_data,
                        })),
                    },
                    _ => continue, // Skip other formats for this benchmark
                };

                let mut total_duration = std::time::Duration::ZERO;
                let mut min_duration = std::time::Duration::MAX;
                let mut max_duration = std::time::Duration::ZERO;

                // Run NUM_RUNS iterations
                for _ in 0..NUM_RUNS {
                    let mut compressor = Compressor::new()?;
                    compressor.set_quality(JPEG_QUALITY)?;

                    let start = Instant::now();
                    let _result = rgb_to_jpeg(&image_raw, &mut compressor)?;
                    let duration = start.elapsed();

                    total_duration += duration;
                    min_duration = min_duration.min(duration);
                    max_duration = max_duration.max(duration);
                }

                let avg_duration = total_duration / NUM_RUNS as u32;

                println!("{} format:", format_name.to_uppercase());
                println!("  Average: {:?}", avg_duration);
                println!("  Min:     {:?}", min_duration);
                println!("  Max:     {:?}", max_duration);
                println!("  Total:   {:?}", total_duration);
                println!();
            } else {
                println!("Skipping {} format - file {} not found", format_name, filename);
            }
        }

        Ok(())
    }
}
