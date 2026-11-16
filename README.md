# reify

reify turns a Spotify track link into a printable music card. Paste a link, pick a cassette or CD cover format, customize it, and export a PNG.

## Features

- Cassette and CD cover card formats
- Fetches the track title and album art from Spotify's oEmbed endpoint
- Scannable Spotify code with five styles, including real alpha transparency
- The code can be dragged freely or snapped to presets
- Custom cover image support for CD covers
- PNG export at 1x, 2x, 3x, or 4x scale
- 100% Rust native desktop app built with eframe and egui

## Requirements

- Windows 10 or 11
- Rust with a stable toolchain (edition 2021)
- A file named `icon.ico` in the project root (used for the window icon and the executable icon)

No fonts need to be downloaded or installed. At startup reify loads fonts from the Windows fonts folder and falls back to the fonts bundled with egui.

## Build and run

```bash
cargo run
```

```bash
cargo build --release
```

The release binary is written to `target/release/reify.exe`.

## How it works

1. Paste a Spotify track link such as `https://open.spotify.com/track/...` or `spotify:track:...`.
2. reify fetches the track title, the album art, and a scannable code.
3. Choose the card format (cassette or CD cover) and adjust the layout.
4. Export the card as a PNG.

An internet connection is required at runtime to fetch track info and generate the scannable code.

## Project layout

```
reify/
  Cargo.toml
  Cargo.lock
  build.rs
  icon.ico
  README.md
  src/
    main.rs
    app.rs
    state.rs
    theme.rs
    fonts.rs
    spotify.rs
    scene.rs
    render_screen.rs
    render_export.rs
    code_image.rs
    worker.rs
```

## Tech stack

- eframe and egui for the window and the interface
- ureq for HTTP, with the Windows native TLS backend
- serde and serde_json for parsing the oEmbed response
- image for decoding images and saving PNGs
- tiny-skia for offline card rendering
- fontdue for text rasterization in exports
- rfd for native open and save dialogs
- percent-encoding for encoding the oEmbed request

---

Created by arkanquixote
