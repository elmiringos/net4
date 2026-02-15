use sled::Db;
use thiserror::Error;
use crate::types::{Address, InferenceRequest, NodeProfile, NodeStatus};

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Failed to open database: {0}")]
    Open(String),
    #[error("Read error: {0}")]
    Read(String),
    #[error("Write error: {0}")]
    Write(String),
    #[error("Serialization error: {0}")]
    Serialize(String),
}

fn sled_err(e: sled::Error) -> StorageError {
    StorageError::Read(e.to_string())
}

fn ser_err(e: serde_json::Error) -> StorageError {
    StorageError::Serialize(e.to_string())
}

#[derive(Clone)]
pub struct Storage {
    db: Db,
    nodes: sled::Tree,
    statuses: sled::Tree,
    requests: sled::Tree,
    meta: sled::Tree,
}

impl Storage {
    pub fn open(path: &str) -> Result<Self, StorageError> {
        let db: Db = sled::open(path).map_err(|e| StorageError::Open(e.to_string()))?;

        let nodes = db.open_tree("nodes").map_err(sled_err)?;
        let statuses = db.open_tree("statuses").map_err(sled_err)?;
        let requests = db.open_tree("requests").map_err(sled_err)?;
        let meta = db.open_tree("meta").map_err(sled_err)?;

        Ok(Self { db, nodes, statuses, requests, meta })
    }

    pub fn put_node_profile(&self, address: &Address, profile: &NodeProfile) -> Result<(), StorageError> {
        let value = serde_json::to_vec(profile).map_err(ser_err)?;
        self.nodes.insert(address.as_bytes(), value).map_err(sled_err)?;
        Ok(())
    }

    pub fn get_node_profile(&self, address: &Address) -> Result<Option<NodeProfile>, StorageError> {
        match self.nodes.get(address.as_bytes()).map_err(sled_err)? {
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes).map_err(ser_err)?)),
            None => Ok(None),
        }
    }

    pub fn put_request(&self, request: &InferenceRequest) -> Result<(), StorageError> {
        let value = serde_json::to_vec(request).map_err(ser_err)?;
        self.requests.insert(request.request_id.as_bytes(), value).map_err(sled_err)?;
        Ok(())
    }

    pub fn get_request(&self, request_id: &str) -> Result<Option<InferenceRequest>, StorageError> {
        match self.requests.get(request_id.as_bytes()).map_err(sled_err)? {
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes).map_err(ser_err)?)),
            None => Ok(None),
        }
    }

    pub fn put_node_status(&self, address: &Address, status: &NodeStatus) -> Result<(), StorageError> {
        let value = serde_json::to_vec(status).map_err(ser_err)?;
        self.statuses.insert(address.as_bytes(), value).map_err(sled_err)?;
        Ok(())
    }

    pub fn get_node_status(&self, address: &Address) -> Result<Option<NodeStatus>, StorageError> {
        match self.statuses.get(address.as_bytes()).map_err(sled_err)? {
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes).map_err(ser_err)?)),
            None => Ok(None),
        }
    }

    pub fn flush(&self) -> Result<(), StorageError> {
        self.db.flush().map_err(sled_err)?;
        Ok(())
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<(), StorageError> {
        self.meta.insert(key.as_bytes(), value.as_bytes()).map_err(sled_err)?;
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> Result<Option<String>, StorageError> {
        match self.meta.get(key.as_bytes()).map_err(sled_err)? {
            Some(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).to_string())),
            None => Ok(None),
        }
    }
}
