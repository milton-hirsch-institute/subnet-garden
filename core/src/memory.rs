// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

pub mod space;
mod subspace;
#[cfg(test)]
mod tests;
mod util;

use crate::errors::{CreateError, DeleteError};
use cidr::IpCidr;
use space::MemorySpace;
use std::collections::BTreeMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct MemorySubnetGarden {
    spaces: BTreeMap<String, MemorySpace>,
}

impl MemorySubnetGarden {
    pub fn new() -> Self {
        MemorySubnetGarden {
            spaces: BTreeMap::new(),
        }
    }
    fn space_count(&self) -> usize {
        self.spaces.len()
    }
    fn new_space(&mut self, name: &str, cidr: IpCidr) -> crate::CreateResult<&mut MemorySpace> {
        if self.spaces.contains_key(name) {
            return Err(CreateError::DuplicateObject);
        }
        let space = MemorySpace::new(cidr);
        self.spaces.insert(name.to_string(), space);
        // Need to figure out how to return this without multiple lookup.
        return Ok(self.spaces.get_mut(name).unwrap());
    }

    fn delete_space(&mut self, name: &str) -> crate::DeleteResult<()> {
        match self.spaces.remove(name) {
            Some(_) => Ok(()),
            None => Err(DeleteError::NoSuchObject),
        }
    }

    fn space_mut(&mut self, name: &str) -> Option<&mut MemorySpace> {
        return self.spaces.get_mut(name);
    }

    fn space_names(&self) -> Vec<String> {
        self.spaces.keys().cloned().collect()
    }

    fn spaces(&self) -> Vec<&MemorySpace> {
        self.spaces.values().collect()
    }

    fn entries(&self) -> Vec<(String, &MemorySpace)> {
        self.spaces
            .iter()
            .map(|(name, space)| (name.clone(), space))
            .collect()
    }
}
