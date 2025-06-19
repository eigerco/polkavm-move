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
    ) -> Result<Vec<u8>, ProgramError>;

    /// Check if a global value exists at the specified address with the given type.
    fn exists(&mut self, address: MoveAddress, typ: StructTagHash) -> Result<bool, ProgramError>;
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

impl Storage for GlobalStorage {
    fn store(
        &mut self,
        address: MoveAddress,
        typ: StructTagHash,
        value: Vec<u8>,
    ) -> Result<(), ProgramError> {
        debug!(
            "Storing global value of type {:?} at address {:?}",
            typ, address
        );

        let key = Key::new(address, typ);

        // Check if the address already exists
        if self.storage.contains_key(&key) {
            debug!(
                "Global already exists at address {address:?} with type {:?}",
                typ
            );
            return Err(ProgramError::MemoryAccess(format!(
                "global already exists at address {address:?} with type {:?}",
                typ
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
        typ: StructTagHash,
        remove: bool,
    ) -> Result<Vec<u8>, ProgramError> {
        debug!(
            "Loading global value of type {:?} at address {:?}",
            typ, address
        );

        let key = Key::new(address, typ);
        let mut value = self
            .storage
            .get(&key)
            .ok_or_else(|| ProgramError::MemoryAccess(format!("global not found at {address:?}")))?
            .clone();
        if remove {
            self.storage.remove(&key);
        } else {
            value.borrow_count += 1;
        }
        debug!("storage: {:x?}", &self.storage);

        Ok(value.data.clone())
    }

    /// Check if a global value exists at the specified address with the given type.
    fn exists(&mut self, address: MoveAddress, typ: StructTagHash) -> Result<bool, ProgramError> {
        debug!(
            "Exists global value of type {:?} at address {:?}",
            typ, address
        );

        let key = Key::new(address, typ);
        let value = self.storage.contains_key(&key);
        debug!("storage: {:x?}", &self.storage);
        Ok(value)
    }
}
