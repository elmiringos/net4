mod types;
mod identity;
mod node;
mod storage;

use types::{Address, ModelInfo, NodeProfile};
use identity::NodeIdentity;
use storage::Storage;

fn main() {
    println!("=== net4 stack test ===\n");

    // 1. Generate identity
    let id = NodeIdentity::generate();
    println!("Generated identity:");
    println!("  address: {}", id.address);
    println!("  pubkey:  {}", hex::encode(id.verifying_key.as_bytes()));

    // 2. Test sign/verify
    let msg = b"hello net4";
    let sig = id.sign(msg);
    let valid = identity::verify(id.verifying_key.as_bytes(), msg, &sig);
    println!("\nSignature test:");
    println!("  message:  \"hello net4\"");
    println!("  valid:    {}", valid);

    // 3. Create a dummy NodeProfile
    let profile = NodeProfile {
        address: id.address,
        gpu_name: "RTX 3090".to_string(),
        vram_total_mb: 24576,
        supported_models: vec![
            ModelInfo {
                model_id: "llama-3-8b".to_string(),
                max_context_length: 8192,
                quantization: "q4_0".to_string(),
            },
            ModelInfo {
                model_id: "mistral-7b".to_string(),
                max_context_length: 32768,
                quantization: "q8_0".to_string(),
            },
        ],
    };

    println!("\nCreated profile:");
    println!("  gpu:    {}", profile.gpu_name);
    println!("  vram:   {} MB", profile.vram_total_mb);
    println!("  models: {}", profile.supported_models.len());

    // 4. Store it
    let db_path = "/tmp/net4_test_db";
    let store = Storage::open(db_path).expect("failed to open storage");

    store.put_node_profile(&id.address, &profile).expect("failed to store profile");
    println!("\nStored profile to {}", db_path);

    // 5. Read it back
    let loaded = store
        .get_node_profile(&id.address)
        .expect("failed to read profile")
        .expect("profile not found");

    println!("\nLoaded profile back:");
    println!("  address: {}", loaded.address);
    println!("  gpu:     {}", loaded.gpu_name);
    println!("  vram:    {} MB", loaded.vram_total_mb);
    for m in &loaded.supported_models {
        println!("  model:   {} (ctx={}, quant={})", m.model_id, m.max_context_length, m.quantization);
    }

    // 6. Verify round-trip
    assert_eq!(profile.address, loaded.address);
    assert_eq!(profile.gpu_name, loaded.gpu_name);
    assert_eq!(profile.vram_total_mb, loaded.vram_total_mb);
    assert_eq!(profile.supported_models.len(), loaded.supported_models.len());

    println!("\n=== all checks passed ===");

    // Cleanup
    drop(store);
    std::fs::remove_dir_all(db_path).ok();
}
