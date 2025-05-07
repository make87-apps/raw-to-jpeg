# RGB-to-JPEG Converter

This application listens for incoming `ImageRgb888` messages, compresses each frame using TurboJPEG, and publishes the
result as an `ImageJpeg` message.

## 📦 Features

- Receives raw RGB888 image frames
- Compresses each frame using libjpeg-turbo (`turbojpeg` crate)
- Publishes JPEG-compressed frames to the make87 message bus
- Reuses a single JPEG compressor for performance
- JPEG quality is configurable via a config value

## 🔧 Configuration

| Name           | Required | Default | Description                           |
|----------------|----------|---------|---------------------------------------|
| `JPEG_QUALITY` | No       | `90`    | JPEG quality (0–100, higher = better) |

## 📥 Input

Subscribes to the `RGB_FRAME` topic and expects messages of type `ImageRgb888`.

## 📤 Output

Publishes to the `JPEG_FRAME` topic as `ImageJpeg` messages. Each message retains the original header and includes the
JPEG-compressed image data.

## 💡 Notes

- Compression is done with a persistent `Compressor` to reduce allocation overhead.
- The app uses `receive_async()` and does not buffer or drop frames.
- For 4K input images, each JPEG output is typically 300–800 KiB depending on quality.
- This app is intended for one-to-one conversion — it does not perform resizing, scaling, or additional image
  preprocessing.

---

© make87, 2025
