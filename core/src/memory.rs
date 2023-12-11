// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests;

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

    fn find_free_space(&mut self, host_length: model::Bits) -> Option<&mut Self> {
        if host_length > self.host_length() {
            return None;
        }
        if self.state == State::Free {
            if host_length == self.host_length() {
                return Some(self);
            } else {
                self.split();
            }
        }
        if self.state == State::Unavailable {
            let found_low = self.low.as_deref_mut()?.find_free_space(host_length);
            match found_low {
                Some(_) => return found_low,
                None => {
                    return self.high.as_deref_mut()?.find_free_space(host_length);
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
    names: HashMap<String, IpCidr>,
}

impl model::Space for Space {
    fn cidr(&self) -> &IpCidr {
        &self.root.cidr
    }

    fn get(&self, host: &str) -> Option<IpCidr> {
        self.names.get(host).copied()
    }

    fn allocate(&mut self, bits: Bits, name: Option<&str>) -> model::AllocateResult<IpCidr> {
        match self.root.find_free_space(bits) {
            Some(subspace) => {
                let new_name: String;
                match name {
                    Some(name) => {
                        if self.names.contains_key(name) {
                            return Err(AllocateError::DuplicateName);
                        }
                        new_name = name.to_string();
                    }
                    None => {
                        new_name = format!("{}", subspace.cidr);
                    }
                }
                if self.names.contains_key(&new_name) {
                    return Err(AllocateError::DuplicateName);
                }
                self.names.insert(new_name, subspace.cidr);
                subspace.state = State::Allocated;
                Ok(subspace.cidr)
            }
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
        if self.names.contains_key(format!("{}", cidr).as_str()) {
            return Err(AllocateError::DuplicateName);
        }
        if self.root.claim(cidr) {
            return Ok(());
        }
        return Err(AllocateError::NoSpaceAvailable);
    }

    fn names(&self) -> Vec<String> {
        self.names.keys().cloned().collect()
    }

    fn cidrs(&self) -> Vec<IpCidr> {
        self.names.values().map(|&cidr| cidr.clone()).collect()
    }

    fn entries(&self) -> Vec<(String, IpCidr)> {
        self.names
            .iter()
            .map(|(name, cidr)| (name.clone(), cidr.clone()))
            .collect()
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
            names: HashMap::new(),
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
