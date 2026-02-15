use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address([u8; 32]);

impl Address {
    pub fn zero() -> Self {
        Address([0u8; 32])
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Address(bytes)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex = hex::encode(&self.0[..8]);
        write!(f, "{}..", hex)
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub max_context_length: u32,
    pub quantization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeProfile {
    pub address: Address,
    pub gpu_name: String,
    pub vram_total_mb: u32,
    pub supported_models: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelThroughput {
    pub model_id: String,
    pub tokens_per_second: f32,
    pub avg_latency_ms: u64,
    pub samples: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub address: Address,
    pub loaded_models: Vec<String>,
    pub active_jobs: u32,
    pub max_concurrent_jobs: u32,
    pub vram_used_mb: u32,
    pub throughput: Vec<ModelThroughput>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub request_id: String,
    pub requester: Address,
    pub model_id: String,
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub request_id: String,
    pub worker: Address,
    pub text: String,
    pub tokens_generated: u32,
    pub execution_time_ms: u64,
    pub tokens_per_second: f32,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestStatus {
    Pending,
    Assigned(Address),
    Running,
    Completed,
    Failed(String),
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerScore {
    pub address: Address,
    pub model_loaded: bool,
    pub estimated_wait_ms: u64,
    pub throughput_tps: f32,
    pub queue_depth: u32,
    pub vram_available_mb: u32,
    pub score: f32,
}
