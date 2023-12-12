// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests;

use crate::errors::{AllocateError, CreateError, RemoveError};
use crate::model;
use crate::model::{AllocateResult, Bits, Space};
use cidr::IpCidr;
use cidr_utils::separator as cidr_separator;
use serde::ser::SerializeMap;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum State {
    Allocated,
    Free,
    Unavailable,
}

fn max_bits(cidr: &IpCidr) -> Bits {
    match cidr {
        IpCidr::V4(_) => 32,
        IpCidr::V6(_) => 128,
    }
}

struct Subspace {
    cidr: IpCidr,
    name: Option<String>,
    high: Option<Box<Self>>,
    low: Option<Box<Self>>,
    state: State,
}

impl Subspace {
    fn new(cidr: IpCidr) -> Self {
        Subspace {
            cidr,
            name: None,
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

    fn find_free_space(&mut self, host_length: Bits) -> Option<&mut Self> {
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
            return match found_low {
                Some(_) => found_low,
                None => self.high.as_deref_mut()?.find_free_space(host_length),
            };
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

pub struct MemorySpace {
    root: Subspace,
    names: HashMap<String, IpCidr>,
}

impl MemorySpace {
    fn new(cidr: IpCidr) -> Self {
        MemorySpace {
            root: Subspace::new(cidr),
            names: HashMap::new(),
        }
    }

    fn list_allocated_subspaces(&self) -> Vec<&Subspace> {
        let mut subspaces = Vec::new();
        let mut stack = Vec::new();
        stack.push(&self.root);
        while !stack.is_empty() {
            let subspace = stack.pop().unwrap();
            match subspace.state {
                State::Allocated => subspaces.push(subspace),
                State::Free => {}
                State::Unavailable => {
                    stack.push(subspace.high.as_deref().unwrap());
                    stack.push(subspace.low.as_deref().unwrap());
                }
            }
        }
        return subspaces;
    }
}

impl Space for MemorySpace {
    fn cidr(&self) -> &IpCidr {
        &self.root.cidr
    }

    fn find_by_name(&self, name: &str) -> Option<IpCidr> {
        self.names.get(name).copied()
    }

    fn allocate(&mut self, bits: Bits, name: Option<&str>) -> AllocateResult<IpCidr> {
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
                subspace.name = Some(new_name.clone());
                self.names.insert(new_name, subspace.cidr);
                subspace.state = State::Allocated;
                Ok(subspace.cidr)
            }
            None => Err(AllocateError::NoSpaceAvailable),
        }
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

    fn cidrs(&self) -> Vec<&IpCidr> {
        let allocated_subspaces = self.list_allocated_subspaces();
        let mut cidrs = Vec::new();
        cidrs.reserve(allocated_subspaces.len());
        for subspace in allocated_subspaces {
            cidrs.push(&subspace.cidr);
        }
        cidrs
    }

    fn entries(&self) -> Vec<(Option<String>, IpCidr)> {
        let allocated_subspaces = self.list_allocated_subspaces();
        let mut entries = Vec::new();
        entries.reserve(allocated_subspaces.len());
        for subspace in allocated_subspaces {
            entries.push((subspace.name.clone(), subspace.cidr));
        }
        entries
    }
}
impl serde::Serialize for MemorySpace {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut entries = self.entries();
        entries.sort_by(|(name1, _), (name2, _)| name1.cmp(name2));
        let mut map = serializer.serialize_map(None)?;
        for (name, cidr) in entries {
            let cidr_string = cidr.to_string();
            map.serialize_entry(&name, &cidr_string)?;
        }
        map.end()
    }
}

pub struct MemorySubnetGarden {
    spaces: HashMap<String, MemorySpace>,
}

impl MemorySubnetGarden {
    pub fn new() -> Self {
        MemorySubnetGarden {
            spaces: HashMap::new(),
        }
    }
}

impl model::SubnetGarden for MemorySubnetGarden {
    fn space_count(&self) -> usize {
        self.spaces.len()
    }
    fn new_space(&mut self, name: &str, cidr: IpCidr) -> model::CreateResult<&mut dyn Space> {
        if self.spaces.contains_key(name) {
            return Err(CreateError::DuplicateObject);
        }
        let space = MemorySpace::new(cidr);
        self.spaces.insert(name.to_string(), space);
        return Ok(self.spaces.get_mut(name).unwrap());
    }

    fn remove_space(&mut self, name: &str) -> model::RemoveResult<()> {
        match self.spaces.remove(name) {
            Some(_) => Ok(()),
            None => Err(RemoveError::NoSuchObject),
        }
    }

    fn space_mut(&mut self, name: &str) -> Option<&mut dyn Space> {
        match self.spaces.get_mut(name) {
            Some(space) => Some(space),
            None => None,
        }
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
