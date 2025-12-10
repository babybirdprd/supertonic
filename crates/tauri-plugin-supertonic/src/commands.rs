use crate::error::{Error, Result};
use crate::SupertonicState;
use std::fs;
use std::path::PathBuf;
use supertonic_tts::{
    load_text_to_speech_from_memory, load_voice_style_from_bytes, write_wav_file, ModelBytes,
};
use tauri::{AppHandle, Manager, Runtime, State};

/// Get the assets directory - works in both dev and production
fn get_assets_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
    // In production, resources are bundled and resolved via the resource directory
    // In dev mode, Tauri doesn't bundle resources, so we need to find the source assets

    // Try production path first (bundled resources)
    if let Ok(resource_dir) = app.path().resource_dir() {
        if resource_dir.join("onnx").exists() {
            return Ok(resource_dir);
        }
    }

    // Dev mode: find assets relative to the executable or workspace
    // The executable is in target/debug, so we go up to find assets
    if let Ok(exe_path) = std::env::current_exe() {
        // Try: exe -> target/debug -> target -> workspace -> assets
        let mut current = exe_path.parent();
        for _ in 0..5 {
            if let Some(dir) = current {
                let assets_path = dir.join("assets");
                if assets_path.join("onnx").exists() {
                    return Ok(assets_path);
                }
                current = dir.parent();
            }
        }
    }

    // Fallback: try current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let assets_path = cwd.join("assets");
        if assets_path.join("onnx").exists() {
            return Ok(assets_path);
        }
        // Try going up from cwd (e.g., if cwd is examples/tauri-app)
        if let Some(parent) = cwd.parent() {
            if let Some(grandparent) = parent.parent() {
                let assets_path = grandparent.join("assets");
                if assets_path.join("onnx").exists() {
                    return Ok(assets_path);
                }
            }
        }
    }

    Err(Error::State("Could not find assets directory. In dev mode, make sure you're running from the workspace root or assets/ exists.".to_string()))
}

/// Helper to read a resource file (handles dev mode and production)
fn read_resource<R: Runtime>(app: &AppHandle<R>, resource_path: &str) -> Result<Vec<u8>> {
    let assets_dir = get_assets_dir(app)?;
    let path = assets_dir.join(resource_path);
    fs::read(&path).map_err(Error::Io)
}

/// List of available voices
#[derive(serde::Serialize)]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
}

/// Initialize response with available voices
#[derive(serde::Serialize)]
pub struct InitResponse {
    pub success: bool,
    pub sample_rate: i32,
    pub available_voices: Vec<VoiceInfo>,
}

/// Initialize the TTS engine with bundled resources
#[tauri::command]
pub async fn initialize<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, SupertonicState>,
) -> Result<InitResponse> {
    // Load ONNX models from bundled resources
    let config_bytes = read_resource(&app, "onnx/tts.json")?;
    let dp_bytes = read_resource(&app, "onnx/duration_predictor.onnx")?;
    let text_enc_bytes = read_resource(&app, "onnx/text_encoder.onnx")?;
    let vector_est_bytes = read_resource(&app, "onnx/vector_estimator.onnx")?;
    let vocoder_bytes = read_resource(&app, "onnx/vocoder.onnx")?;
    let unicode_indexer_bytes = read_resource(&app, "onnx/unicode_indexer.json")?;

    let models = ModelBytes {
        config: &config_bytes,
        duration_predictor: &dp_bytes,
        text_encoder: &text_enc_bytes,
        vector_estimator: &vector_est_bytes,
        vocoder: &vocoder_bytes,
        unicode_indexer: &unicode_indexer_bytes,
    };

    let engine = load_text_to_speech_from_memory(models, false).map_err(Error::Supertonic)?;
    let sample_rate = engine.sample_rate;

    *state.engine.lock().unwrap() = Some(engine);

    // Discover available voice styles
    let assets_dir = get_assets_dir(&app)?;
    let voices_dir = assets_dir.join("voice_styles");

    let mut available_voices = Vec::new();
    if let Ok(entries) = fs::read_dir(&voices_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    let id = name.trim_end_matches(".json").to_string();
                    available_voices.push(VoiceInfo {
                        id: id.clone(),
                        name: id,
                    });
                }
            }
        }
    }

    Ok(InitResponse {
        success: true,
        sample_rate,
        available_voices,
    })
}

/// Set the active voice style
#[tauri::command]
pub async fn set_voice<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, SupertonicState>,
    voice_id: String,
) -> Result<()> {
    let voice_path = format!("voice_styles/{}.json", voice_id);
    let voice_bytes = read_resource(&app, &voice_path)?;

    let byte_slices = vec![voice_bytes.as_slice()];
    let style = load_voice_style_from_bytes(&byte_slices, false).map_err(Error::Supertonic)?;

    *state.style.lock().unwrap() = Some(style);

    Ok(())
}

/// Legacy: Load engine from custom path (for development/testing)
#[tauri::command]
pub async fn load_engine<R: Runtime>(
    _app: AppHandle<R>,
    state: State<'_, SupertonicState>,
    onnx_dir: String,
) -> Result<()> {
    let base_path = PathBuf::from(&onnx_dir);

    let config_bytes = fs::read(base_path.join("tts.json")).map_err(Error::Io)?;
    let dp_bytes = fs::read(base_path.join("duration_predictor.onnx")).map_err(Error::Io)?;
    let text_enc_bytes = fs::read(base_path.join("text_encoder.onnx")).map_err(Error::Io)?;
    let vector_est_bytes = fs::read(base_path.join("vector_estimator.onnx")).map_err(Error::Io)?;
    let vocoder_bytes = fs::read(base_path.join("vocoder.onnx")).map_err(Error::Io)?;
    let unicode_indexer_bytes =
        fs::read(base_path.join("unicode_indexer.json")).map_err(Error::Io)?;

    let models = ModelBytes {
        config: &config_bytes,
        duration_predictor: &dp_bytes,
        text_encoder: &text_enc_bytes,
        vector_estimator: &vector_est_bytes,
        vocoder: &vocoder_bytes,
        unicode_indexer: &unicode_indexer_bytes,
    };

    let engine = load_text_to_speech_from_memory(models, false).map_err(Error::Supertonic)?;
    *state.engine.lock().unwrap() = Some(engine);

    Ok(())
}

/// Legacy: Load voice from custom paths
#[tauri::command]
pub async fn load_voice<R: Runtime>(
    _app: AppHandle<R>,
    state: State<'_, SupertonicState>,
    voice_paths: Vec<String>,
) -> Result<()> {
    let mut bytes_buffers = Vec::new();
    for path in &voice_paths {
        bytes_buffers.push(fs::read(path).map_err(Error::Io)?);
    }

    let byte_slices: Vec<&[u8]> = bytes_buffers.iter().map(|b| b.as_slice()).collect();
    let style = load_voice_style_from_bytes(&byte_slices, false).map_err(Error::Supertonic)?;

    *state.style.lock().unwrap() = Some(style);

    Ok(())
}

/// Response from speak command
#[derive(serde::Serialize)]
pub struct SpeakResponse {
    pub audio: Vec<f32>,
    pub duration: f32,
    pub sample_rate: i32,
}

#[tauri::command]
pub async fn speak<R: Runtime>(
    _app: AppHandle<R>,
    state: State<'_, SupertonicState>,
    text: String,
    speed: Option<f32>,
    silence_duration: Option<f32>,
    total_step: Option<usize>,
) -> Result<SpeakResponse> {
    let mut engine_guard = state.engine.lock().unwrap();
    let engine = engine_guard.as_mut().ok_or(Error::State(
        "Engine not initialized. Call 'initialize' first.".to_string(),
    ))?;

    let style_guard = state.style.lock().unwrap();
    let style = style_guard.as_ref().ok_or(Error::State(
        "No voice selected. Call 'set_voice' first.".to_string(),
    ))?;

    let sample_rate = engine.sample_rate;
    let (audio, duration) = engine
        .call(
            &text,
            style,
            total_step.unwrap_or(10),
            speed.unwrap_or(1.0),
            silence_duration.unwrap_or(0.2),
        )
        .map_err(Error::Supertonic)?;

    Ok(SpeakResponse {
        audio,
        duration,
        sample_rate,
    })
}

/// Response from batch speak command
#[derive(serde::Serialize)]
pub struct BatchSpeakResponse {
    pub audio_list: Vec<Vec<f32>>,
    pub durations: Vec<f32>,
    pub sample_rate: i32,
}

#[tauri::command]
pub async fn speak_batch<R: Runtime>(
    _app: AppHandle<R>,
    state: State<'_, SupertonicState>,
    texts: Vec<String>,
    speed: Option<f32>,
    total_step: Option<usize>,
) -> Result<BatchSpeakResponse> {
    let mut engine_guard = state.engine.lock().unwrap();
    let engine = engine_guard
        .as_mut()
        .ok_or(Error::State("Engine not initialized".to_string()))?;

    let style_guard = state.style.lock().unwrap();
    let style = style_guard
        .as_ref()
        .ok_or(Error::State("No voice selected".to_string()))?;

    let sample_rate = engine.sample_rate;
    let (audio_list, durations) = engine
        .batch(
            &texts,
            style,
            total_step.unwrap_or(10),
            speed.unwrap_or(1.0),
        )
        .map_err(Error::Supertonic)?;

    Ok(BatchSpeakResponse {
        audio_list,
        durations,
        sample_rate,
    })
}

/// Engine info response
#[derive(serde::Serialize)]
pub struct EngineInfo {
    pub initialized: bool,
    pub voice_loaded: bool,
    pub sample_rate: Option<i32>,
}

#[tauri::command]
pub async fn get_engine_info<R: Runtime>(
    _app: AppHandle<R>,
    state: State<'_, SupertonicState>,
) -> Result<EngineInfo> {
    let engine_guard = state.engine.lock().unwrap();
    let style_guard = state.style.lock().unwrap();

    Ok(EngineInfo {
        initialized: engine_guard.is_some(),
        voice_loaded: style_guard.is_some(),
        sample_rate: engine_guard.as_ref().map(|e| e.sample_rate),
    })
}

#[tauri::command]
pub async fn save_wav<R: Runtime>(
    _app: AppHandle<R>,
    state: State<'_, SupertonicState>,
    audio: Vec<f32>,
    output_path: String,
) -> Result<()> {
    let engine_guard = state.engine.lock().unwrap();
    let engine = engine_guard
        .as_ref()
        .ok_or(Error::State("Engine not initialized".to_string()))?;

    write_wav_file(&output_path, &audio, engine.sample_rate)
        .map_err(|e| Error::State(format!("Failed to write WAV: {}", e)))?;

    Ok(())
}
