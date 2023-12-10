// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::errors::{AllocateError, CreateError, RemoveError};
use crate::model;
use std::collections::HashMap;

use crate::model::{AllocateResult, Bits};
use cidr::IpCidr;
use cidr_utils::separator as cidr_separator;

#[derive(Debug, PartialEq)]
enum State {
    Allocated,
    Free,
    Unavailable,
}

struct Subspace {
    cidr: IpCidr,
    high: Option<Box<Self>>,
    low: Option<Box<Self>>,
    state: State,
}

fn max_bits(cidr: &IpCidr) -> model::Bits {
    match cidr {
        IpCidr::V4(_) => 32,
        IpCidr::V6(_) => 128,
    }
}

impl Subspace {
    fn new(cidr: IpCidr) -> Self {
        Subspace {
            cidr,
            high: None,
            low: None,
            state: State::Free,
        }
    }
    fn host_length(self: &Self) -> Bits {
        return max_bits(&self.cidr) - self.cidr.network_length();
    }

    fn split(self: &mut Self) {
        self.state = State::Unavailable;
        let new_network_length = self.cidr.network_length() + 1;
        let low_cidr: IpCidr;
        let high_cidr: IpCidr;
        match self.cidr {
            IpCidr::V4(cidr) => {
                let subnets =
                    cidr_separator::Ipv4CidrSeparator::sub_networks(&cidr, new_network_length);
                let subnets_vec = subnets.unwrap();
                low_cidr = IpCidr::V4(*subnets_vec.get(0).unwrap());
                high_cidr = IpCidr::V4(*subnets_vec.get(1).unwrap());
            }
            IpCidr::V6(cidr) => {
                let subnets =
                    cidr_separator::Ipv6CidrSeparator::sub_networks(&cidr, new_network_length);
                let subnets_vec = subnets.unwrap();
                low_cidr = IpCidr::V6(*subnets_vec.get(0).unwrap());
                high_cidr = IpCidr::V6(*subnets_vec.get(1).unwrap());
            }
        }
        self.low = Some(Box::new(Subspace::new(low_cidr)));
        self.high = Some(Box::new(Subspace::new(high_cidr)));
    }

    fn find_space(&mut self, host_length: model::Bits) -> Option<&mut Self> {
        if host_length > self.host_length() {
            return None;
        }
        if self.state == State::Free {
            if host_length == self.host_length() {
                self.state = State::Allocated;
                return Some(self);
            } else {
                self.split();
            }
        }
        if self.state == State::Unavailable {
            let found_low = self.low.as_deref_mut()?.find_space(host_length);
            match found_low {
                Some(_) => return found_low,
                None => {
                    return self.high.as_deref_mut()?.find_space(host_length);
                }
            }
        }
        return None;
    }

    fn claim(&mut self, cidr: &IpCidr) -> bool {
        let first = cidr.first_address();
        let last = cidr.last_address();
        if !(self.cidr.contains(&first) && self.cidr.contains(&last)) {
            return false;
        }

        match self.state {
            State::Allocated => return false,
            State::Free => {
                if self.cidr == *cidr {
                    self.state = State::Allocated;
                    return true;
                }
                self.split();
            }
            State::Unavailable => {}
        }

        if self.low.as_deref_mut().unwrap().claim(cidr) {
            return true;
        }

        self.high.as_deref_mut().unwrap().claim(cidr)
    }
}

struct Space {
    root: Subspace,
}

impl model::Space for Space {
    fn cidr(&self) -> &IpCidr {
        &self.root.cidr
    }

    fn allocate(&mut self, bits: Bits) -> model::AllocateResult<&IpCidr> {
        match self.root.find_space(bits) {
            Some(subspace) => Ok(&subspace.cidr),
            None => Err(AllocateError::NoSpaceAvailable),
        }
    }

    fn list_cidrs(&self) -> Vec<&IpCidr> {
        let mut cidrs = Vec::new();
        let mut stack = Vec::new();
        stack.push(&self.root);
        while !stack.is_empty() {
            let subspace = stack.pop().unwrap();
            match subspace.state {
                State::Allocated => cidrs.push(&subspace.cidr),
                State::Free => {}
                State::Unavailable => {
                    stack.push(subspace.high.as_deref().unwrap());
                    stack.push(subspace.low.as_deref().unwrap());
                }
            }
        }
        return cidrs;
    }

    fn claim(&mut self, cidr: &IpCidr) -> AllocateResult<()> {
        if self.root.claim(cidr) {
            return Ok(());
        }
        return Err(AllocateError::NoSpaceAvailable);
    }
}

pub struct Memory {
    spaces: HashMap<String, Space>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            spaces: HashMap::new(),
        }
    }
}

impl model::SubnetGarden for Memory {
    fn space_count(&self) -> usize {
        self.spaces.len()
    }
    fn new_space(
        &mut self,
        name: &str,
        cidr: IpCidr,
    ) -> model::CreateResult<&mut dyn model::Space> {
        if self.spaces.contains_key(name) {
            return Err(CreateError::DuplicateObject);
        }
        let space = Space {
            root: Subspace::new(cidr),
        };
        self.spaces.insert(name.to_string(), space);
        return Ok(self.spaces.get_mut(name).unwrap());
    }

    fn remove_space(&mut self, name: &str) -> model::RemoveResult<()> {
        match self.spaces.remove(name) {
            Some(_) => Ok(()),
            None => Err(RemoveError::NoSuchObject),
        }
    }

    fn space_mut(&mut self, name: &str) -> Option<&mut dyn model::Space> {
        match self.spaces.get_mut(name) {
            Some(space) => Some(space),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SubnetGarden;
    use cidr::{IpCidr, Ipv4Cidr, Ipv6Cidr};
    use std::net::{Ipv4Addr, Ipv6Addr};

    static TEST_CIDR4: IpCidr = IpCidr::V4(match Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 16) {
        Ok(cidr) => cidr,
        _ => panic!("Failed to create test v4 cidr"),
    });

    static TEST_CIDR6: IpCidr = IpCidr::V6(
        match Ipv6Cidr::new(Ipv6Addr::new(1, 2, 3, 4, 10, 20, 0, 0), 112) {
            Ok(cidr) => cidr,
            _ => panic!("Failed to create test v6 cidr"),
        },
    );

    fn new_test_space() -> Memory {
        let mut instance = Memory::new();
        instance.new_space("test4", TEST_CIDR4).unwrap();
        instance.new_space("test6", TEST_CIDR6).unwrap();
        return instance;
    }

    mod memory {
        use super::*;

        #[test]
        fn new_memory_garden() {
            let instance = Memory::new();
            assert_eq!(instance.space_count(), 0);
        }

        #[test]
        fn new_space_duplicate_object() {
            let mut instance = new_test_space();
            let result = instance.new_space("test4", TEST_CIDR4);
            assert_eq!(result.err(), Some(CreateError::DuplicateObject));
        }

        #[test]
        fn new_space_success() {
            let instance = new_test_space();
            assert_eq!(instance.space_count(), 2);
        }

        #[test]
        fn remove_space_no_such_object() {
            let mut instance = new_test_space();
            let result = instance.remove_space("does-not-exist");
            assert_eq!(result.err(), Some(RemoveError::NoSuchObject));
        }

        #[test]
        fn remove_space_success() {
            let mut instance = new_test_space();
            instance.remove_space("test4").unwrap();
            assert_eq!(instance.space_count(), 1);
        }

        #[test]
        fn space_success() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            assert_eq!(*space.cidr(), TEST_CIDR4);
        }

        #[test]
        fn space_not_found() {
            let mut instance = new_test_space();
            assert!(instance.space_mut("does-not-exist").is_none());
        }
    }

    mod space {
        use super::*;

        #[test]
        fn too_many_bits() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let result = space.allocate(17);
            assert_eq!(result.err(), Some(AllocateError::NoSpaceAvailable));
        }

        #[test]
        fn no_space_available() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            space.allocate(16).unwrap();
            let result = space.allocate(16);
            assert_eq!(result.err(), Some(AllocateError::NoSpaceAvailable));
        }
        #[test]
        fn allocate_success_v4() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let result = space.allocate(4).unwrap();
            assert_eq!(
                result,
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap())
            );
        }
        #[test]
        fn allocate_multi_sizes() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let result1 = space.allocate(4).unwrap();
            assert_eq!(
                result1,
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap())
            );
            let result2 = space.allocate(8).unwrap();
            assert_eq!(
                result2,
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 1, 0), 24).unwrap())
            );
        }

        #[test]
        fn allocate_success_v6() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test6").unwrap();
            let result = space.allocate(4).unwrap();
            assert_eq!(
                result,
                &IpCidr::V6(Ipv6Cidr::new(Ipv6Addr::new(1, 2, 3, 4, 10, 20, 0, 0), 124).unwrap())
            );
        }

        #[test]
        fn max_bits() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let result = space.allocate(16).unwrap();
            assert_eq!(
                result,
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 16).unwrap())
            );
        }

        #[test]
        fn list_empty_cidrs() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let cidrs = space.list_cidrs();
            assert_eq!(cidrs.len(), 0);
        }
        #[test]
        fn list_cidrs() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            space.allocate(4).unwrap();
            space.allocate(5).unwrap();
            space.allocate(5).unwrap();
            space.allocate(4).unwrap();
            space.allocate(4).unwrap();
            space.allocate(4).unwrap();
            let cidrs = space.list_cidrs();
            assert_eq!(cidrs.len(), 6);
            assert_eq!(
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap()),
                cidrs[0],
            );
            assert_eq!(
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 16), 28).unwrap()),
                cidrs[1],
            );
            assert_eq!(
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 32), 27).unwrap()),
                cidrs[2],
            );
            assert_eq!(
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 64), 27).unwrap()),
                cidrs[3],
            );
            assert_eq!(
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 96), 28).unwrap()),
                cidrs[4],
            );
            assert_eq!(
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 112), 28).unwrap()),
                cidrs[5],
            );
        }
        #[test]
        fn claim_out_of_range() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 21, 0, 0), 28).unwrap());
            let result = space.claim(&cidr);
            assert_eq!(result, Err(AllocateError::NoSpaceAvailable));
        }

        #[test]
        fn claim_already_claimed() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
            space.claim(&cidr).unwrap();
            let result = space.claim(&cidr);
            assert_eq!(result, Err(AllocateError::NoSpaceAvailable));
        }

        #[test]
        fn claim_already_allocated() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
            space.allocate(16).unwrap();
            let result = space.claim(&cidr);
            assert_eq!(result, Err(AllocateError::NoSpaceAvailable));
        }

        #[test]
        fn claim_success() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
            let result = space.claim(&cidr);
            assert_eq!(result, Ok(()));
        }
    }
}
