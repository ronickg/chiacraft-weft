//! A store that keeps entropy in memory only (tests).
use super::{SecretStore, StoreKind};
use crate::{Result, secret::Secret};
use std::{collections::BTreeMap, sync::Mutex};

#[derive(Default)]
pub struct MemoryStore {
    items: Mutex<BTreeMap<u32, Secret<Vec<u8>>>>,
}
impl SecretStore for MemoryStore {
    fn kind(&self) -> StoreKind {
        StoreKind::Memory
    }
    fn put(&self, account: u32, entropy: &Secret<Vec<u8>>) -> Result<()> {
        self.items
            .lock()
            .expect("store")
            .insert(account, entropy.duplicate());
        Ok(())
    }
    fn get(&self, account: u32) -> Result<Option<Secret<Vec<u8>>>> {
        Ok(self
            .items
            .lock()
            .expect("store")
            .get(&account)
            .map(Secret::duplicate))
    }
    fn delete(&self, account: u32) -> Result<()> {
        self.items.lock().expect("store").remove(&account);
        Ok(())
    }
}
