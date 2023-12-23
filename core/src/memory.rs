// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

mod space;
mod subspace;
#[cfg(test)]
mod tests;
mod util;

use crate::errors::{CreateError, RemoveError};
use crate::Space;
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
}

impl crate::SubnetGarden for MemorySubnetGarden {
    fn space_count(&self) -> usize {
        self.spaces.len()
    }
    fn new_space(&mut self, name: &str, cidr: IpCidr) -> crate::CreateResult<&mut dyn Space> {
        if self.spaces.contains_key(name) {
            return Err(CreateError::DuplicateObject);
        }
        let space = MemorySpace::new(cidr);
        self.spaces.insert(name.to_string(), space);
        // Need to figure out how to return this without multiple lookup.
        return Ok(self.spaces.get_mut(name).unwrap());
    }

    fn delete_space(&mut self, name: &str) -> crate::RemoveResult<()> {
        match self.spaces.remove(name) {
            Some(_) => Ok(()),
            None => Err(RemoveError::NoSuchObject),
        }
    }

    fn space_mut(&mut self, name: &str) -> Option<&mut dyn Space> {
        return self
            .spaces
            .get_mut(name)
            .map(|space| space as &mut dyn Space);
    }

    fn space_names(&self) -> Vec<String> {
        self.spaces.keys().cloned().collect()
    }

    fn spaces(&self) -> Vec<&dyn Space> {
        self.spaces
            .values()
            .map(|space| space as &dyn Space)
            .collect()
    }

    fn entries(&self) -> Vec<(String, &dyn Space)> {
        self.spaces
            .iter()
            .map(|(name, space)| (name.clone(), space as &dyn Space))
            .collect()
    }
}
