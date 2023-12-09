// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::errors::{CreateError, RemoveError};
use crate::model;
use std::collections::HashMap;

use cidr::IpCidr;

struct Space {
    cidr: IpCidr,
}

impl model::Space for Space {
    fn cidr(&self) -> &IpCidr {
        &self.cidr
    }
}

struct Memory {
    spaces: HashMap<String, Space>,
}

impl Memory {
    fn new() -> Self {
        Memory {
            spaces: HashMap::new(),
        }
    }
}

impl model::SubnetGarden for Memory {
    fn space_count(&self) -> usize {
        self.spaces.len()
    }
    fn new_space(&mut self, name: &str, cidr: &IpCidr) -> model::CreateResult<&dyn model::Space> {
        if self.spaces.contains_key(name) {
            return Err(CreateError::DuplicateObject);
        }
        let space = Space { cidr: cidr.clone() };
        self.spaces.insert(name.to_string(), space);
        return Ok(self.spaces.get_mut(name).unwrap());
    }

    fn remove_space(&mut self, name: &str) -> model::RemoveResult<()> {
        match self.spaces.remove(name) {
            Some(_) => Ok(()),
            None => Err(RemoveError::NoSuchObject),
        }
    }

    fn space(&self, name: &str) -> Option<&dyn model::Space> {
        match self.spaces.get(name) {
            Some(space) => Some(space),
            None => None,
        }
    }
}

mod tests {
    use super::*;
    use crate::model::SubnetGarden;
    use cidr::{IpCidr, Ipv4Cidr};
    use std::net::Ipv4Addr;

    static TEST_CIDR4: IpCidr = IpCidr::V4(match Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 16) {
        Ok(cidr) => cidr,
        _ => panic!("Failed to create test cidr"),
    });

    #[test]
    fn new_garden() {
        let instance = Memory::new();
        assert_eq!(instance.space_count(), 0);
    }
    #[test]
    fn new_space_duplicate_object() {
        let mut instance = Memory::new();
        instance.new_space("test", &TEST_CIDR4).unwrap();
        let result = instance.new_space("test", &TEST_CIDR4);
        match result {
            Err(CreateError::DuplicateObject) => (),
            _ => panic!("Expected duplicate object error"),
        }
    }
    #[test]
    fn new_space_success() {
        let mut instance = Memory::new();
        instance.new_space("test", &TEST_CIDR4).unwrap();
        assert_eq!(instance.space_count(), 1);
    }
    #[test]
    fn remove_space_no_such_object() {
        let mut instance = Memory::new();
        let result = instance.remove_space("test");
        match result {
            Err(RemoveError::NoSuchObject) => (),
            _ => panic!("Expected no such object error"),
        }
    }
    #[test]
    fn remove_space_success() {
        let mut instance = Memory::new();
        instance.new_space("test", &TEST_CIDR4).unwrap();
        instance.remove_space("test").unwrap();
        assert_eq!(instance.space_count(), 0);
    }
    #[test]
    fn space_success() {
        let mut instance = Memory::new();
        instance.new_space("test", &TEST_CIDR4).unwrap();
        let space = instance.space("test").unwrap();
        assert_eq!(*space.cidr(), TEST_CIDR4);
    }
    #[test]
    fn space_not_found() {
        let instance = Memory::new();
        match instance.space("test") {
            None => (),
            _ => panic!("Expected space not found"),
        }
    }
}
