// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests;

use crate::errors::{AllocateError, RenameError};
use crate::subspace::{State, Subspace};
use crate::{AllocateResult, Bits, CidrRecord, RenameResult};
use cidr::IpCidr;
use serde::ser::SerializeStruct;
use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub struct SubnetGarden {
    root: Subspace,
    names: HashMap<String, IpCidr>,
}

impl SubnetGarden {
    pub fn new(cidr: IpCidr) -> Self {
        SubnetGarden {
            root: Subspace::new(cidr),
            names: HashMap::new(),
        }
    }

    fn list_allocated_subspaces(&self) -> Vec<&Subspace> {
        let mut subspaces = Vec::new();
        let mut stack = Vec::new();
        stack.push(&self.root);
        while let Some(subspace) = stack.pop() {
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
    pub fn cidr(&self) -> &IpCidr {
        &self.root.cidr
    }

    pub fn find_by_name(&self, name: &str) -> Option<IpCidr> {
        self.names.get(name).copied()
    }

    pub fn allocate(&mut self, bits: Bits, name: Option<&str>) -> AllocateResult<IpCidr> {
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

    pub fn free(&mut self, cidr: &IpCidr) -> bool {
        return self.root.free(cidr);
    }

    pub fn claim(&mut self, cidr: &IpCidr, name: Option<&str>) -> AllocateResult<()> {
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

    pub fn rename(&mut self, cidr: &IpCidr, name: Option<&str>) -> RenameResult<()> {
        // Find record that is being renamed
        let subspace: &mut Subspace = match self.root.find_record(cidr) {
            Some(record) => record,
            None => return Err(RenameError::NoSuchObject),
        };

        // Ignore if name is not changing
        let name_as_string = match name {
            Some(name) => Some(name.to_string()),
            None => None,
        };
        if subspace.name.as_deref() == name {
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

    pub fn names(&self) -> Vec<String> {
        self.names.keys().cloned().collect()
    }

    pub fn cidrs(&self) -> Vec<&IpCidr> {
        let allocated_subspaces = self.list_allocated_subspaces();
        let mut cidrs = Vec::new();
        cidrs.reserve(allocated_subspaces.len());
        for subspace in allocated_subspaces {
            cidrs.push(&subspace.cidr);
        }
        cidrs
    }

    pub fn entries(&self) -> Vec<crate::CidrRecord> {
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

impl serde::Serialize for SubnetGarden {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut space = serializer.serialize_struct("MemorySpace", 2)?;
        space.serialize_field("cidr", &self.root.cidr.to_string())?;
        let mut entries = self.entries();
        entries.sort_by(|record1, record2| record1.cmp(record2));
        space.serialize_field("subnets", &entries)?;

        space.end()
    }
}

impl<'s> serde::Deserialize<'s> for SubnetGarden {
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

        fn load_cidrs(
            entries: &Vec<CidrRecord>,
            cidr: &IpCidr,
        ) -> Result<SubnetGarden, AllocateError> {
            let mut space = SubnetGarden::new(*cidr);
            for entry in entries {
                let entry_name = entry.name.as_deref();
                space.claim(&entry.cidr, entry_name)?;
            }
            Ok(space)
        }
        struct MemorySpaceVisitor;
        impl<'s> serde::de::Visitor<'s> for MemorySpaceVisitor {
            type Value = SubnetGarden;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct MemorySpace")
            }
            fn visit_seq<V>(self, mut seq: V) -> Result<SubnetGarden, V::Error>
            where
                V: serde::de::SeqAccess<'s>,
            {
                let cidr = seq
                    .next_element::<&str>()?
                    .ok_or_else(|| serde::de::Error::missing_field("cidr"))?;
                let cidr = cidr.parse::<IpCidr>().map_err(serde::de::Error::custom)?;
                let entries = seq
                    .next_element::<Vec<crate::CidrRecord>>()?
                    .ok_or_else(|| serde::de::Error::missing_field("subnets"))?;

                Ok(load_cidrs(&entries, &cidr).map_err(serde::de::Error::custom)?)
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
                            let cidr_string = map.next_value::<String>()?;
                            cidr = Some(
                                cidr_string
                                    .parse::<IpCidr>()
                                    .map_err(serde::de::Error::custom)?,
                            );
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
                Ok(load_cidrs(&subnets, &cidr).map_err(serde::de::Error::custom)?)
            }
        }
        const FIELDS: &[&str] = &["cidr", "subnets"];
        deserializer.deserialize_struct("MemorySpace", FIELDS, MemorySpaceVisitor)
    }
}
