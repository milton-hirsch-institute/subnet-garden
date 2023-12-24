// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
pub(crate) mod tests;

use crate::errors::{CreateError, DeleteError};
use crate::space::SubnetGarden;
use cidr::IpCidr;
use std::collections::BTreeMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct SubnetGardenLibrary {
    spaces: BTreeMap<String, SubnetGarden>,
}

impl SubnetGardenLibrary {
    pub fn new() -> Self {
        SubnetGardenLibrary {
            spaces: BTreeMap::new(),
        }
    }
    pub fn space_count(&self) -> usize {
        self.spaces.len()
    }
    pub fn new_space(
        &mut self,
        name: &str,
        cidr: IpCidr,
    ) -> crate::CreateResult<&mut SubnetGarden> {
        if self.spaces.contains_key(name) {
            return Err(CreateError::DuplicateObject);
        }
        let space = SubnetGarden::new(cidr);
        self.spaces.insert(name.to_string(), space);
        // Need to figure out how to return this without multiple lookup.
        return Ok(self.spaces.get_mut(name).unwrap());
    }

    pub fn delete_space(&mut self, name: &str) -> crate::DeleteResult<()> {
        match self.spaces.remove(name) {
            Some(_) => Ok(()),
            None => Err(DeleteError::NoSuchObject),
        }
    }

    pub fn space_mut(&mut self, name: &str) -> Option<&mut SubnetGarden> {
        return self.spaces.get_mut(name);
    }

    pub fn space_names(&self) -> Vec<String> {
        self.spaces.keys().cloned().collect()
    }

    pub fn spaces(&self) -> Vec<&SubnetGarden> {
        self.spaces.values().collect()
    }

    pub fn entries(&self) -> Vec<(String, &SubnetGarden)> {
        self.spaces
            .iter()
            .map(|(name, space)| (name.clone(), space))
            .collect()
    }
}
