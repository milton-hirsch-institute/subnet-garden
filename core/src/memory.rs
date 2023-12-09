// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::errors::CreateError;
use crate::model;
use crate::model::CreateResult;
use std::collections::HashMap;

use cidr::IpCidr;

struct Space {
    cidr: IpCidr,
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
    fn new_space(&mut self, name: &str, cidr: &IpCidr) -> CreateResult<()> {
        if self.spaces.contains_key(name) {
            return Err(CreateError::DuplicateObject);
        }
        let space = Space { cidr: cidr.clone() };
        self.spaces.insert(name.to_string(), space);
        Ok(())
    }

    fn space_cidr(&self, name: &str) -> Option<&IpCidr> {
        match self.spaces.get(name) {
            Some(space) => Some(&space.cidr),
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
    fn space_cidr_success() {
        let mut instance = Memory::new();
        instance.new_space("test", &TEST_CIDR4).unwrap();
        assert_eq!(instance.space_cidr("test"), Some(&TEST_CIDR4));
    }
    #[test]
    fn space_cidr_not_found() {
        let instance = Memory::new();
        assert_eq!(instance.space_cidr("test"), None);
    }
}
