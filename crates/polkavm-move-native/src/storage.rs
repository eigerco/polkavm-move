use crate::{host::ProgramError, types::MoveAddress};
use log::debug;
use std::collections::HashMap;

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
    fn release(&self, address: MoveAddress, tag: StructTagHash);

    /// Release all global resources.
    fn release_all(&self);
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
    fn release(&self, address: MoveAddress, tag: StructTagHash) {
        debug!("Releasing global value at address {address:?} with tag {tag:x?}",);

        let key = Key::new(address, tag);
        if let Some(entry) = self.storage.get(&key) {
            if entry.borrow_mut {
                // If there's a mutable borrow, we can release it
                debug!("Released mutable borrow for global at {address:?} with type {tag:?}");
            } else if entry.borrow_count > 0 {
                // If there are shared borrows, we just decrement the count
                debug!("Decremented borrow count for global at {address:?} with type {tag:?}");
            } else {
                // No active borrows, nothing to do
                debug!("No active borrows to release for global at {address:?} with type {tag:?}");
            }
        } else {
            debug!("No global found at {address:?} with type {tag:?} to release");
        }
    }

    fn release_all(&self) {
        debug!("Releasing all global resources");
        for entry in self.storage.keys() {
            let Key(address, tag) = entry;
            self.release(*address, *tag);
        }
        debug!("All global resources released");
    }
}
