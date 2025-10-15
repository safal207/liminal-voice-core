use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use liminal_voice_core::device_memory::{self, DeviceMemoryStore};

fn temp_file_path(label: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!(
        "liminal_voice_core_{}_{}_{}.jsonl",
        label,
        std::process::id(),
        nanos
    ));
    path
}

#[test]
fn load_missing_file_returns_empty() {
    let path = temp_file_path("missing");
    let path_str = path.to_string_lossy().to_string();
    if path.exists() {
        let _ = fs::remove_file(&path);
    }

    let store = DeviceMemoryStore::load(&path_str);
    assert!(store.data.is_empty());
    assert_eq!(store.path, path_str);
}

#[test]
fn update_and_persist_device_memory() {
    let path = temp_file_path("persist");
    let path_str = path.to_string_lossy().to_string();
    if path.exists() {
        let _ = fs::remove_file(&path);
    }

    let mut store = DeviceMemoryStore::load(&path_str);
    store.update("Phone", 1.0, 60.0, 0.7, 0.2, 0.8);
    store.update("Phone", 1.2, 70.0, 0.8, 0.3, 0.7);
    store.save();

    let metadata = fs::metadata(&path_str).expect("memory file metadata");
    assert!(metadata.len() > 0);

    let text = fs::read_to_string(&path_str).expect("memory file contents");
    assert!(text.contains("Phone"));

    let store_reload = DeviceMemoryStore::load(&path_str);
    let memory =
        device_memory::suggest_profile(&store_reload, "Phone").expect("persisted device profile");

    assert_eq!(memory.sessions, 2);
    assert!((memory.avg_pace - 1.1).abs() < 1e-3);
    assert!((memory.avg_pause - 65.0).abs() < 1e-3);
    assert!((memory.avg_articulation - 0.75).abs() < 1e-3);
    assert!((memory.avg_drift - 0.25).abs() < 1e-3);
    assert!((memory.avg_res - 0.75).abs() < 1e-3);

    let _ = fs::remove_file(&path_str);
}
