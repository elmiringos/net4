use rocksdb::{DB, Options, ColumnFamilyDescriptor};
use thiserror::Error;
use std::sync::Arc;
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

const CF_NODES: &str = "nodes";
const CF_REQUESTS: &str = "requests";
const CF_META: &str = "meta";

#[derive(Clone)]
pub struct Storage {
    db: Arc<DB>,
}

impl Storage {
    pub fn open(path: &str) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cfs = vec![
            ColumnFamilyDescriptor::new(CF_NODES, Options::default()),
            ColumnFamilyDescriptor::new(CF_REQUESTS, Options::default()),
            ColumnFamilyDescriptor::new(CF_META, Options::default()),
        ];

        let db = DB::open_cf_descriptors(&opts, path, cfs).map_err(|e| StorageError::Open(e.to_string()))?;
        let arc_db = Arc::new(db);

        Ok(Self { db: arc_db } )
    }

    pub fn put_node_profile(&self, address: &Address, profile: &NodeProfile) -> Result<(), StorageError> {
        let cf = self.db.cf_handle(CF_NODES)
            .ok_or(StorageError::Read("nodes cf not found".to_string()))?;

        let key = address.as_bytes();
        let value = serde_json::to_vec(profile)
            .map_err(|e| StorageError::Serialize(e.to_string()))?;

        self.db.put_cf(&cf, key, value)
            .map_err(|e| StorageError::Write(e.to_string()))?;

        Ok(())
    }

    pub fn get_node_profile(&self, address: &Address) -> Result<Option<NodeProfile>, StorageError> {
        let cf = self.db.cf_handle(CF_NODES)
            .ok_or(StorageError::Read("nodes cf not found".to_string()))?;

        let key = address.as_bytes();

        match self.db.get_cf(&cf, key).map_err(|e| StorageError::Read(e.to_string()))? {
            Some(bytes) => {
                let profile = serde_json::from_slice(&bytes)
                    .map_err(|e| StorageError::Serialize(e.to_string()))?;
                Ok(Some(profile))
            }
            None => Ok(None),
        }
    }

    pub fn put_request(&self, request: &InferenceRequest) -> Result<(), StorageError> {
        let cf = self.db.cf_handle(CF_REQUESTS)
            .ok_or(StorageError::Read("requests cf not found".to_string()))?;
        let key = request.request_id.as_bytes();
        let value = serde_json::to_vec(request)
            .map_err(|e| StorageError::Serialize(e.to_string()))?;
        self.db.put_cf(&cf, key, value)
            .map_err(|e| StorageError::Write(e.to_string()))?;
        Ok(())
    }

    pub fn get_request(&self, request_id: &str) -> Result<Option<InferenceRequest>, StorageError> {
        let cf = self.db.cf_handle(CF_REQUESTS)
            .ok_or(StorageError::Read("requests cf not found".to_string()))?;
        let key = request_id.as_bytes();
        match self.db.get_cf(&cf, key).map_err(|e| StorageError::Read(e.to_string()))? {
            Some(bytes) => {
                let request = serde_json::from_slice(&bytes)
                    .map_err(|e| StorageError::Serialize(e.to_string()))?;
                Ok(Some(request))
            }
            None => Ok(None),
        }
    }

    pub fn put_node_status(&self, address: &Address, status: &NodeStatus) -> Result<(), StorageError> {
            let cf = self.db.cf_handle(CF_NODES)
                .ok_or(StorageError::Read("nodes cf not found".to_string()))?;
            // Different key prefix so it doesn't overwrite the profile
            let mut key = Vec::from(b"status:" as &[u8]);
            key.extend_from_slice(address.as_bytes());
            let value = serde_json::to_vec(status)
                .map_err(|e| StorageError::Serialize(e.to_string()))?;
            self.db.put_cf(&cf, key, value)
                .map_err(|e| StorageError::Write(e.to_string()))?;
            Ok(())
        }

    pub fn get_node_status(&self, address: &Address) -> Result<Option<NodeStatus>, StorageError> {
        let cf = self.db.cf_handle(CF_NODES)
            .ok_or(StorageError::Read("nodes cf not found".to_string()))?;
        let mut key = Vec::from(b"status:" as &[u8]);
        key.extend_from_slice(address.as_bytes());
        match self.db.get_cf(&cf, key).map_err(|e| StorageError::Read(e.to_string()))? {
            Some(bytes) => {
                let status = serde_json::from_slice(&bytes)
                    .map_err(|e| StorageError::Serialize(e.to_string()))?;
                Ok(Some(status))
            }
            None => Ok(None),
        }
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let cf = self.db.cf_handle(CF_META)
            .ok_or(StorageError::Read("meta cf not found".to_string()))?;
        self.db.put_cf(&cf, key.as_bytes(), value.as_bytes())
            .map_err(|e| StorageError::Write(e.to_string()))?;
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> Result<Option<String>, StorageError> {
        let cf = self.db.cf_handle(CF_META)
            .ok_or(StorageError::Read("meta cf not found".to_string()))?;
        match self.db.get_cf(&cf, key.as_bytes()).map_err(|e| StorageError::Read(e.to_string()))? {
            Some(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).to_string())),
            None => Ok(None),
        }
    }
}
