extern crate alloc;

use crate::{host::ProgramError, types::MoveAddress};
use alloc::{format, vec::Vec};
use hashbrown::HashMap;
use log::debug;

pub type StructTagHash = [u8; 32];

pub trait Storage {
    /// Store a global value at the specified address with the given type.
    fn store(
        &mut self,
        address: MoveAddress,
        typ: StructTagHash,
        value: Vec<u8>,
    ) -> Result<(), ProgramError>;

    /// Load a global value from the specified address with the given type.
    fn load(
        &mut self,
        address: MoveAddress,
        typ: StructTagHash,
        remove: bool,
        is_mut: bool,
    ) -> Result<Vec<u8>, ProgramError>;

    /// Check if a global value exists at the specified address with the given type.
    fn exists(&mut self, address: MoveAddress, typ: StructTagHash) -> Result<bool, ProgramError>;

    /// Release a global value at the specified address with the given tag.
    fn release(&mut self, address: MoveAddress, tag: StructTagHash);

    /// Release all global resources.
    fn release_all(&mut self);

    fn is_borrowed(&self, move_signer: MoveAddress, tag: StructTagHash) -> bool;

    fn update(
        &mut self,
        address: MoveAddress,
        typ: StructTagHash,
        value: Vec<u8>,
    ) -> Result<(), ProgramError> {
        debug!("Updating global value of type {typ:x?} at address {address:?}");
        self.store(address, typ, value)
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct Key(MoveAddress, StructTagHash);

impl Key {
    /// Create a new key from a Move address and type.
    pub fn new(address: MoveAddress, typ: StructTagHash) -> Self {
        Self(address, typ)
    }
}

#[derive(Debug, Clone)]
struct GlobalResourceEntry {
    /// The serialized resource contents (Move struct instance).
    pub data: Vec<u8>,

    /// Number of active shared borrows (`&T`).
    pub borrow_count: u32,

    /// True if there's an active mutable borrow (`&mut T`).
    pub borrow_mut: bool,
}

impl GlobalResourceEntry {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            borrow_count: 0,
            borrow_mut: false,
        }
    }
}

pub struct GlobalStorage {
    storage: HashMap<Key, GlobalResourceEntry>,
}

impl GlobalStorage {
    /// Create a new global storage instance.
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
}

impl Default for GlobalStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage for GlobalStorage {
    fn store(
        &mut self,
        address: MoveAddress,
        tag: StructTagHash,
        value: Vec<u8>,
    ) -> Result<(), ProgramError> {
        debug!("Storing global value of type {tag:x?} at address {address:?}",);

        let key = Key::new(address, tag);

        // Check if the address already exists
        if self.storage.contains_key(&key) {
            return Err(ProgramError::MemoryAccess(format!(
                "global already exists at address {address:?} with type {tag:x?}",
            )));
        }

        // Store the value in the storage map
        self.storage.insert(key, GlobalResourceEntry::new(value));
        debug!("storage: {:x?}", &self.storage);

        Ok(())
    }

    /// Update a global value at the specified address with the given type.
    fn update(
        &mut self,
        address: MoveAddress,
        tag: StructTagHash,
        value: Vec<u8>,
    ) -> Result<(), ProgramError> {
        debug!("Storing global value of type {tag:x?} at address {address:?}",);

        let key = Key::new(address, tag);

        let entry = self.storage.get(&key).ok_or_else(|| {
            ProgramError::MemoryAccess(format!("global not found at {address:?}"))
        })?;
        if entry.borrow_mut {
            // update the value in the storage map if it was mutably borrowed
            self.storage.insert(key, GlobalResourceEntry::new(value));
        }

        debug!("updated storage: {:x?}", &self.storage);
        Ok(())
    }

    /// Load a global value from the specified address with the given type.
    fn load(
        &mut self,
        address: MoveAddress,
        tag: StructTagHash,
        remove: bool,
        is_mut: bool,
    ) -> Result<Vec<u8>, ProgramError> {
        debug!("Loading global value of type {tag:x?} at address {address:?}, is_mut: {is_mut}, remove: {remove}",);

        let key = Key::new(address, tag);
        let value = self.storage.get_mut(&key).ok_or_else(|| {
            ProgramError::MemoryAccess(format!("global not found at {address:?}"))
        })?;
        let rv = value.data.clone();
        if remove {
            self.storage.remove(&key);
        } else {
            if value.borrow_mut {
                return Err(ProgramError::MemoryAccess(format!(
                    "mutable borrow already exists for global at {address:?} with type {tag:?}",
                )));
            }
            if is_mut {
                if value.borrow_count > 0 {
                    return Err(ProgramError::MemoryAccess(format!(
                        "cannot create mutable borrow for global at {address:?} with type {tag:?} while there are active shared borrows",
                    )));
                }
                value.borrow_mut = true;
            }
            value.borrow_count += 1;
        }
        debug!("storage: {:x?}", &self.storage);

        Ok(rv)
    }

    /// Check if a global value exists at the specified address with the given type.
    fn exists(&mut self, address: MoveAddress, tag: StructTagHash) -> Result<bool, ProgramError> {
        debug!("Exists global value of type {tag:x?} at address {address:?}",);

        let key = Key::new(address, tag);
        let value = self.storage.contains_key(&key);
        debug!("Global exists: {value}");
        debug!("storage: {:x?}", &self.storage);
        Ok(value)
    }

    /// Release a global value at the specified address with the given tag.
    fn release(&mut self, address: MoveAddress, tag: StructTagHash) {
        debug!("Releasing global value at address {address:?} with tag {tag:x?}",);

        let key = Key::new(address, tag);
        if let Some(entry) = self.storage.get_mut(&key) {
            if entry.borrow_mut {
                // If there's a mutable borrow, we can release it
                debug!("Released mutable borrow for global at {address:?} with type {tag:?}");
                entry.borrow_mut = false;
            }
            if entry.borrow_count > 0 {
                // If there are shared borrows, we just decrement the count
                debug!("Decremented borrow count for global at {address:?} with type {tag:?}");
                entry.borrow_count -= 1;
            } else {
                // No active borrows, nothing to do
                debug!("No active borrows to release for global at {address:?} with type {tag:?}");
            }
        } else {
            debug!("No global found at {address:?} with type {tag:?} to release");
        }
        debug!("storage: {:x?}", &self.storage);
    }

    fn release_all(&mut self) {
        debug!("Releasing all global resources");
        let keys: Vec<(MoveAddress, StructTagHash)> = self
            .storage
            .keys()
            .map(|Key(addr, tag)| (*addr, *tag))
            .collect();
        for entry in keys {
            let (address, tag) = entry;
            self.release(address, tag);
        }
        debug!("All global resources released");
    }

    fn is_borrowed(&self, address: MoveAddress, tag: StructTagHash) -> bool {
        let key = Key::new(address, tag);
        if let Some(entry) = self.storage.get(&key) {
            entry.borrow_count > 0 || entry.borrow_mut
        } else {
            false
        }
    }
}
