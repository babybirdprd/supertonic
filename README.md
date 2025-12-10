# Supertonic

**High-performance, on-device Text-to-Speech synthesis in Rust.**

Supertonic is a blazing-fast TTS engine powered by ONNX Runtime, designed for edge deployment. Run neural TTS models locally on desktops and mobile devices with minimal dependencies.

---

## ✨ Features

- **🚀 ONNX Runtime** — Leverage optimized inference with hardware acceleration
- **📱 Cross-Platform** — Desktop (Windows, macOS, Linux) and Mobile (Android, iOS) via Tauri
- **🔊 High-Quality Output** — Neural TTS with configurable voice styles
- **⚡ Fast Inference** — Optimized array processing with ndarray and rayon
- **🎛️ Flexible API** — Use as a Rust library, CLI tool, or Tauri plugin

---

## 📦 Crates

| Crate | Description |
|-------|-------------|
| [`supertonic-tts`](./crates/core) | Core TTS engine library with ONNX inference |
| [`tauri-plugin-supertonic`](./crates/tauri-plugin-supertonic) | Tauri v2 plugin for desktop & mobile apps |

---

## 🚀 Quick Start

### CLI Usage

```bash
# Build the CLI
cargo build --release -p supertonic-tts

# Run TTS synthesis
./target/release/tts \
  --onnx-dir assets/onnx \
  --voice-style assets/voice_styles/M1.json \
  --text "Hello, world! This is Supertonic speaking."
```

### CLI Options

| Option | Default | Description |
|--------|---------|-------------|
| `--onnx-dir` | `assets/onnx` | Directory containing ONNX models |
| `--voice-style` | `assets/voice_styles/M1.json` | Voice style JSON file(s) |
| `--text` | (sample text) | Text to synthesize |
| `--speed` | `1.05` | Speech speed factor |
| `--total-step` | `5` | Denoising steps (higher = better quality) |
| `--save-dir` | `results` | Output directory for WAV files |
| `--batch` | `false` | Enable batch mode for multiple texts |
| `--use-gpu` | `false` | Use GPU for inference |

---

## 📚 Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
supertonic-tts = { path = "crates/core" }
```

### Basic Example

```rust
use supertonic_tts::{load_text_to_speech, load_voice_style, write_wav_file};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the TTS engine
    let mut tts = load_text_to_speech("assets/onnx", false)?;
    
    // Load a voice style
    let style = load_voice_style(&["assets/voice_styles/M1.json".to_string()], true)?;
    
    // Generate speech
    let (audio, duration) = tts.call(
        "Hello from Supertonic!",
        &style,
        10,    // denoising steps
        1.0,   // speed
        0.2,   // silence between chunks
    )?;
    
    // Save to WAV
    write_wav_file("output.wav", &audio, tts.sample_rate)?;
    
    println!("Generated {:.2}s of audio", duration);
    Ok(())
}
```

### Loading from Memory (for Mobile/Embedded)

```rust
use supertonic_tts::{ModelBytes, load_text_to_speech_from_memory, load_voice_style_from_bytes};

// Load model bytes (e.g., from Android assets)
let models = ModelBytes {
    config: include_bytes!("../assets/onnx/tts.json"),
    duration_predictor: include_bytes!("../assets/onnx/duration_predictor.onnx"),
    text_encoder: include_bytes!("../assets/onnx/text_encoder.onnx"),
    vector_estimator: include_bytes!("../assets/onnx/vector_estimator.onnx"),
    vocoder: include_bytes!("../assets/onnx/vocoder.onnx"),
    unicode_indexer: include_bytes!("../assets/onnx/unicode_indexer.json"),
};

let mut tts = load_text_to_speech_from_memory(models, false)?;
```

---

## 📱 Tauri Plugin

Integrate Supertonic into your Tauri v2 app for cross-platform TTS.

### Installation

In your Tauri app's `Cargo.toml`:

```toml
[dependencies]
tauri-plugin-supertonic = { path = "../crates/tauri-plugin-supertonic" }
```

### Setup

```rust
// src-tauri/src/lib.rs
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_supertonic::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Frontend API

```typescript
import { invoke } from '@tauri-apps/api/core';

// Load the TTS engine
await invoke('plugin:supertonic|load_engine', { 
  onnxDir: '/path/to/onnx/models' 
});

// Load a voice style
await invoke('plugin:supertonic|load_voice', { 
  voicePaths: ['/path/to/voice_style.json'] 
});

// Get engine info
const info = await invoke('plugin:supertonic|get_engine_info', {});
// Returns: { loaded: boolean, sample_rate: number | null }

// Generate speech (single text)
const result = await invoke('plugin:supertonic|speak', { 
  text: 'Hello from Tauri!',
  speed: 1.0,           // Optional, default 1.0
  silenceDuration: 0.2, // Optional, default 0.2
  totalStep: 10         // Optional, default 10 (higher = better quality)
});
// Returns: { audio: number[], duration: number, sample_rate: number }

// Batch TTS (multiple texts at once)
const batchResult = await invoke('plugin:supertonic|speak_batch', { 
  texts: ['First text', 'Second text', 'Third text'],
  speed: 1.0,
  totalStep: 10
});
// Returns: { audio_list: number[][], durations: number[], sample_rate: number }

// Save audio to WAV file
await invoke('plugin:supertonic|save_wav', {
  audio: result.audio,
  outputPath: '/path/to/output.wav'
});
```

### Running the Example App

A fully-featured example Tauri app is included in `examples/tauri-app/` with a modern UI that demonstrates all plugin features:

- 🔧 Engine & voice loading
- 💬 Single text-to-speech
- 📦 Batch TTS mode
- 💾 Save to WAV

```bash
# Prerequisites: Install Tauri CLI
cargo install tauri-cli

# Navigate to the example
cd examples/tauri-app

# Run the desktop app (development mode)
cargo tauri dev

# Build for production
cargo tauri build
```

> **Note**: Update the ONNX directory and voice style paths in the app to point to your `assets/` folder.

#### Mobile (Android)

```bash
# Initialize Android project (first time only)
cargo tauri android init

# Run on Android device/emulator
cargo tauri android dev

# Build Android APK
cargo tauri android build
```

#### Mobile (iOS)

```bash
# Initialize iOS project (first time only, macOS required)
cargo tauri ios init

# Run on iOS simulator
cargo tauri ios dev

# Build iOS app
cargo tauri ios build
```

---

## 📁 Required Assets

Place the following files in your `assets/onnx` directory:

| File | Description |
|------|-------------|
| `tts.json` | Model configuration |
| `duration_predictor.onnx` | Duration prediction model |
| `text_encoder.onnx` | Text encoding model |
| `vector_estimator.onnx` | Denoising/vector estimation model |
| `vocoder.onnx` | Waveform generation model |
| `unicode_indexer.json` | Unicode character mappings |

Voice styles go in `assets/voice_styles/`:

| File | Description |
|------|-------------|
| `M1.json`, `F1.json`, etc. | Voice style embeddings |

---

## 🏗️ Building

```bash
# Build all crates
cargo build --release

# Build only the core library
cargo build --release -p supertonic-tts

# Build the Tauri plugin
cargo build --release -p tauri-plugin-supertonic

# Run tests
cargo test --workspace
```

---

## 📋 Project Structure

```
supertonic/
├── crates/
│   ├── core/                      # Core TTS library
│   │   ├── src/
│   │   │   ├── lib.rs             # Library exports
│   │   │   ├── model.rs           # TTS model & inference
│   │   │   ├── text.rs            # Text processing
│   │   │   ├── audio.rs           # WAV output
│   │   │   ├── config.rs          # Model configuration
│   │   │   ├── error.rs           # Error types
│   │   │   ├── utils.rs           # Utilities
│   │   │   └── bin/tts.rs         # CLI binary
│   │   └── Cargo.toml
│   │
│   └── tauri-plugin-supertonic/   # Tauri v2 plugin
│       ├── src/
│       │   ├── lib.rs             # Plugin initialization
│       │   ├── commands.rs        # Tauri commands
│       │   ├── models.rs          # Request/response types
│       │   └── error.rs           # Error handling
│       └── Cargo.toml
│
├── assets/
│   ├── onnx/                      # ONNX models
│   └── voice_styles/              # Voice style embeddings
│
├── examples/
│   └── tauri-app/                 # Example Tauri application
│
└── Cargo.toml                     # Workspace configuration
```

---

## 🔧 Dependencies

Core dependencies:

- **[ort](https://crates.io/crates/ort)** — ONNX Runtime bindings
- **[ndarray](https://crates.io/crates/ndarray)** — N-dimensional arrays
- **[hound](https://crates.io/crates/hound)** — WAV file I/O
- **[rayon](https://crates.io/crates/rayon)** — Parallel processing

---

## 📄 License

[MIT](LICENSE) or [Apache-2.0](LICENSE-APACHE), at your option.

---

## 🤝 Contributing

Contributions welcome! Please open an issue or PR.
