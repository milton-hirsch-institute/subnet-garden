// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests;

use crate::errors::{AllocateError, RenameError};
use crate::memory::subspace::{State, Subspace};
use crate::{AllocateResult, Bits, RenameResult, Space};
use cidr::IpCidr;
use serde::ser::SerializeStruct;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(PartialEq, Debug)]
pub struct MemorySpace {
    root: Subspace,
    names: HashMap<String, IpCidr>,
}

impl MemorySpace {
    pub(super) fn new(cidr: IpCidr) -> Self {
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
                if let Some(name) = name {
                    if self.names.contains_key(name) {
                        return Err(AllocateError::DuplicateName);
                    }
                    subspace.name = Some(name.to_string());
                    self.names.insert(name.to_string(), subspace.cidr);
                }
                subspace.state = State::Allocated;
                Ok(subspace.cidr)
            }
            None => Err(AllocateError::NoSpaceAvailable),
        }
    }

    fn claim(&mut self, cidr: &IpCidr, name: Option<&str>) -> AllocateResult<()> {
        if let Some(name) = name {
            if self.names.contains_key(name) {
                return Err(AllocateError::DuplicateName);
            }
            self.names.insert(name.to_string(), *cidr);
        }
        if self.root.claim(cidr, name) {
            return Ok(());
        }
        return Err(AllocateError::NoSpaceAvailable);
    }

    fn rename(&mut self, cidr: &IpCidr, name: Option<&str>) -> RenameResult<()> {
        // Find record that is being renamed
        let subspace: &mut Subspace = match self.root.find_record(cidr) {
            Some(record) => record,
            None => return Err(RenameError::NameNotFound),
        };

        // Ignore if name is not changing
        let name_as_string = match name {
            Some(name) => Some(name.to_string()),
            None => None,
        };
        if subspace.name == name_as_string {
            return Ok(());
        }

        // Check that name does not already exist
        if let Some(name) = name {
            if self.names.contains_key(name) {
                return Err(RenameError::DuplicateName);
            }
            self.names.insert(name.to_string(), *cidr);
        }

        // Remove old name
        if let Some(record_name) = &subspace.name {
            self.names.remove(record_name);
        }

        // Update record name
        subspace.name = name_as_string;
        Ok(())
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

    fn entries(&self) -> Vec<crate::CidrRecord> {
        let mut allocated_subspaces = self.list_allocated_subspaces();
        allocated_subspaces.sort_by(|subspace1, subspace2| subspace1.cidr.cmp(&subspace2.cidr));
        let mut entries = Vec::new();
        entries.reserve(allocated_subspaces.len());
        for subspace in allocated_subspaces {
            entries.push(crate::CidrRecord::new(
                subspace.cidr,
                subspace.name.as_deref(),
            ));
        }
        entries
    }
}

impl serde::Serialize for MemorySpace {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut space = serializer.serialize_struct("MemorySpace", 2)?;
        space.serialize_field("cidr", &self.root.cidr.to_string())?;
        let mut entries = self.entries();
        entries.sort_by(|record1, record2| record1.cmp(record2));
        space.serialize_field("subnets", &entries)?;

        space.end()
    }
}

impl<'s> serde::Deserialize<'s> for MemorySpace {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'s>,
    {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Cidr,
            Subnets,
        }
        struct MemorySpaceVisitor;
        impl<'s> serde::de::Visitor<'s> for MemorySpaceVisitor {
            type Value = MemorySpace;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct MemorySpace")
            }
            fn visit_seq<V>(self, mut seq: V) -> Result<MemorySpace, V::Error>
            where
                V: serde::de::SeqAccess<'s>,
            {
                let cidr = seq
                    .next_element::<&str>()?
                    .ok_or_else(|| serde::de::Error::missing_field("cidr"))?;
                let cidr = cidr.parse::<IpCidr>().unwrap();
                let mut space = MemorySpace::new(cidr);
                let entries = seq
                    .next_element::<Vec<crate::CidrRecord>>()?
                    .ok_or_else(|| serde::de::Error::missing_field("subnets"))?;
                for entry in entries {
                    space.claim(&entry.cidr, entry.name.as_deref()).unwrap();
                }
                Ok(space)
            }
            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'s>,
            {
                let mut cidr: Option<IpCidr> = None;
                let mut entries: Option<Vec<crate::CidrRecord>> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Cidr => {
                            if cidr.is_some() {
                                return Err(serde::de::Error::duplicate_field("cidr"));
                            }
                            cidr = match map.next_value::<String>() {
                                Ok(cidr) => match IpCidr::from_str(cidr.as_str()) {
                                    Ok(cidr) => Some(cidr),
                                    Err(err) => return Err(serde::de::Error::custom(err)),
                                },
                                Err(err) => return Err(err),
                            };
                        }
                        Field::Subnets => {
                            if entries.is_some() {
                                return Err(serde::de::Error::duplicate_field("subnets"));
                            }
                            entries = Some(map.next_value()?);
                        }
                    }
                }
                let cidr = cidr.ok_or_else(|| serde::de::Error::missing_field("cidr"))?;
                let subnets = entries.ok_or_else(|| serde::de::Error::missing_field("subnets"))?;
                let mut space = MemorySpace::new(cidr);
                for entry in subnets {
                    let entry_name = entry.name.as_deref();
                    space.claim(&entry.cidr, entry_name).unwrap();
                }
                Ok(space)
            }
        }
        const FIELDS: &[&str] = &["cidr", "subnets"];
        deserializer.deserialize_struct("MemorySpace", FIELDS, MemorySpaceVisitor)
    }
}