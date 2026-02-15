use std::sync::atomic::{AtomicU32, Ordering};
use net4::identity::NodeIdentity;
use net4::storage::Storage;
use net4::types::*;

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn temp_db() -> (Storage, String) {
    let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/net4_test_{}_{}", std::process::id(), n);
    let store = Storage::open(&path).expect("failed to open db");
    (store, path)
}

fn cleanup(path: &str) {
    std::fs::remove_dir_all(path).ok();
}

fn dummy_profile(address: Address) -> NodeProfile {
    NodeProfile {
        address,
        gpu_name: "RTX 4090".to_string(),
        vram_total_mb: 24576,
        supported_models: vec![
            ModelInfo {
                model_id: "llama-3-8b".to_string(),
                max_context_length: 8192,
                quantization: "q4_0".to_string(),
            },
        ],
    }
}

// ── NodeProfile tests ──

#[test]
fn put_and_get_node_profile() {
    let (store, path) = temp_db();
    let id = NodeIdentity::generate();
    let profile = dummy_profile(id.address);

    store.put_node_profile(&id.address, &profile).unwrap();
    let loaded = store.get_node_profile(&id.address).unwrap().unwrap();

    assert_eq!(loaded.address, profile.address);
    assert_eq!(loaded.gpu_name, "RTX 4090");
    assert_eq!(loaded.vram_total_mb, 24576);
    assert_eq!(loaded.supported_models.len(), 1);
    assert_eq!(loaded.supported_models[0].model_id, "llama-3-8b");

    cleanup(&path);
}

#[test]
fn get_missing_profile_returns_none() {
    let (store, path) = temp_db();
    let id = NodeIdentity::generate();

    let result = store.get_node_profile(&id.address).unwrap();
    assert!(result.is_none());

    cleanup(&path);
}

#[test]
fn put_overwrites_existing_profile() {
    let (store, path) = temp_db();
    let id = NodeIdentity::generate();

    let profile1 = dummy_profile(id.address);
    store.put_node_profile(&id.address, &profile1).unwrap();

    let profile2 = NodeProfile {
        gpu_name: "A100".to_string(),
        vram_total_mb: 81920,
        ..profile1
    };
    store.put_node_profile(&id.address, &profile2).unwrap();

    let loaded = store.get_node_profile(&id.address).unwrap().unwrap();
    assert_eq!(loaded.gpu_name, "A100");
    assert_eq!(loaded.vram_total_mb, 81920);

    cleanup(&path);
}

// ── InferenceRequest tests ──

#[test]
fn put_and_get_request() {
    let (store, path) = temp_db();
    let id = NodeIdentity::generate();

    let req = InferenceRequest {
        request_id: "req-001".to_string(),
        requester: id.address,
        model_id: "llama-3-8b".to_string(),
        prompt: "Hello world".to_string(),
        max_tokens: 256,
        temperature: 0.7,
        created_at: 1700000000,
    };

    store.put_request(&req).unwrap();
    let loaded = store.get_request("req-001").unwrap().unwrap();

    assert_eq!(loaded.request_id, "req-001");
    assert_eq!(loaded.requester, id.address);
    assert_eq!(loaded.prompt, "Hello world");
    assert_eq!(loaded.max_tokens, 256);

    cleanup(&path);
}

#[test]
fn get_missing_request_returns_none() {
    let (store, path) = temp_db();
    assert!(store.get_request("nonexistent").unwrap().is_none());
    cleanup(&path);
}

// ── NodeStatus tests ──

#[test]
fn put_and_get_node_status() {
    let (store, path) = temp_db();
    let id = NodeIdentity::generate();

    let status = NodeStatus {
        address: id.address,
        loaded_models: vec!["llama-3-8b".to_string()],
        active_jobs: 2,
        max_concurrent_jobs: 4,
        vram_used_mb: 12000,
        throughput: vec![
            ModelThroughput {
                model_id: "llama-3-8b".to_string(),
                tokens_per_second: 45.5,
                avg_latency_ms: 120,
                samples: 100,
            },
        ],
        timestamp: 1700000000,
    };

    store.put_node_status(&id.address, &status).unwrap();
    let loaded = store.get_node_status(&id.address).unwrap().unwrap();

    assert_eq!(loaded.address, id.address);
    assert_eq!(loaded.active_jobs, 2);
    assert_eq!(loaded.loaded_models, vec!["llama-3-8b"]);
    assert_eq!(loaded.throughput.len(), 1);
    assert_eq!(loaded.throughput[0].tokens_per_second, 45.5);

    cleanup(&path);
}

#[test]
fn profile_and_status_dont_collide() {
    let (store, path) = temp_db();
    let id = NodeIdentity::generate();

    let profile = dummy_profile(id.address);
    store.put_node_profile(&id.address, &profile).unwrap();

    let status = NodeStatus {
        address: id.address,
        loaded_models: vec![],
        active_jobs: 0,
        max_concurrent_jobs: 4,
        vram_used_mb: 0,
        throughput: vec![],
        timestamp: 1700000000,
    };
    store.put_node_status(&id.address, &status).unwrap();

    let loaded_profile = store.get_node_profile(&id.address).unwrap().unwrap();
    let loaded_status = store.get_node_status(&id.address).unwrap().unwrap();

    assert_eq!(loaded_profile.gpu_name, "RTX 4090");
    assert_eq!(loaded_status.active_jobs, 0);

    cleanup(&path);
}

// ── Meta tests ──

#[test]
fn set_and_get_meta() {
    let (store, path) = temp_db();

    store.set_meta("version", "0.1.0").unwrap();
    let val = store.get_meta("version").unwrap().unwrap();
    assert_eq!(val, "0.1.0");

    cleanup(&path);
}

#[test]
fn get_missing_meta_returns_none() {
    let (store, path) = temp_db();
    assert!(store.get_meta("nonexistent").unwrap().is_none());
    cleanup(&path);
}

// ── Flush test ──

#[test]
fn flush_does_not_error() {
    let (store, path) = temp_db();
    let id = NodeIdentity::generate();
    store.put_node_profile(&id.address, &dummy_profile(id.address)).unwrap();
    store.flush().unwrap();
    cleanup(&path);
}
