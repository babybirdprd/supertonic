const COMMANDS: &[&str] = &[
    "initialize",
    "set_voice",
    "load_engine",
    "load_voice",
    "speak",
    "speak_batch",
    "get_engine_info",
    "save_wav",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
