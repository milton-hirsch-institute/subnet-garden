// Copyright 2023-2024 The Milton Hirsch Institute, B.V.
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
pub struct SubnetPool {
    root: Subspace,
    names: HashMap<String, IpCidr>,
}

impl SubnetPool {
    pub fn new(cidr: IpCidr) -> Self {
        SubnetPool {
            root: Subspace::new(cidr),
            names: HashMap::new(),
        }
    }

    fn iter_allocated_subspaces(&self) -> impl Iterator<Item = &Subspace> {
        let mut stack = Vec::new();
        stack.push(&self.root);
        std::iter::from_fn(move || {
            while let Some(subspace) = stack.pop() {
                match subspace.state {
                    State::Allocated => return Some(subspace),
                    State::Free => {}
                    State::Unavailable => {
                        stack.push(subspace.high.as_deref().unwrap());
                        stack.push(subspace.low.as_deref().unwrap());
                    }
                }
            }
            None
        })
    }

    #[inline(always)]
    pub fn allocated_count(&self) -> usize {
        self.root.allocated_count
    }

    #[inline(always)]
    pub fn named_count(&self) -> usize {
        self.names.len()
    }

    #[inline(always)]
    pub fn max_available_bits(&self) -> Bits {
        self.root.max_available_bits
    }

    #[inline(always)]
    pub fn find_by_name(&self, name: &str) -> Option<IpCidr> {
        self.names.get(name).copied()
    }

    pub fn contains(&self, cidr: &IpCidr) -> bool {
        if let Some(subspace) = self.root.find_record(cidr) {
            return subspace.state == State::Allocated;
        }
        false
    }

    pub fn allocate(&mut self, bits: Bits, name: Option<&str>) -> AllocateResult<IpCidr> {
        match self.root.allocate_free_space(bits, name) {
            Some(cidr) => {
                if let Some(name) = name {
                    if self.names.contains_key(name) {
                        return Err(AllocateError::DuplicateName);
                    }
                    self.names.insert(name.to_string(), cidr);
                }
                Ok(cidr)
            }
            None => Err(AllocateError::NoSpaceAvailable),
        }
    }

    pub fn free(&mut self, cidr: &IpCidr) -> bool {
        self.root.free(cidr)
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
        Err(AllocateError::NoSpaceAvailable)
    }

    pub fn rename(&mut self, cidr: &IpCidr, name: Option<&str>) -> RenameResult<()> {
        // Find record that is being renamed
        let subspace: &mut Subspace = match self.root.find_record_mut(cidr) {
            Some(record) => record,
            None => return Err(RenameError::NoSuchObject),
        };

        // Ignore if name is not changing
        if subspace.record.name.as_deref() == name {
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
        if let Some(record_name) = &subspace.record.name {
            self.names.remove(record_name);
        }

        // Update record name
        subspace.record.name = name.map(|name| name.to_string());
        Ok(())
    }

    pub fn names(&self) -> impl Iterator<Item = String> + '_ {
        self.names.keys().map(|name| name.to_string())
    }

    pub fn cidrs(&self) -> impl Iterator<Item = &IpCidr> {
        self.iter_allocated_subspaces()
            .map(|subspace| &subspace.record.cidr)
    }

    pub fn records(&self) -> impl Iterator<Item = &CidrRecord> {
        self.iter_allocated_subspaces()
            .map(|subspace| &subspace.record)
    }
}

impl serde::Serialize for SubnetPool {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut pool = serializer.serialize_struct("SubnetPool", 2)?;
        pool.serialize_field("cidr", &self.root.record.cidr.to_string())?;
        let records: Vec<&CidrRecord> = self.records().collect();
        pool.serialize_field("subnets", &records)?;

        pool.end()
    }
}

impl<'s> serde::Deserialize<'s> for SubnetPool {
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
        ) -> Result<SubnetPool, AllocateError> {
            let mut pool = SubnetPool::new(*cidr);
            for entry in entries {
                let entry_name = entry.name.as_deref();
                pool.claim(&entry.cidr, entry_name)?;
            }
            Ok(pool)
        }
        struct SubnetPoolVisitor;
        impl<'s> serde::de::Visitor<'s> for SubnetPoolVisitor {
            type Value = SubnetPool;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct SubspacePool")
            }
            fn visit_seq<V>(self, mut seq: V) -> Result<SubnetPool, V::Error>
            where
                V: serde::de::SeqAccess<'s>,
            {
                let cidr = seq
                    .next_element::<&str>()?
                    .ok_or_else(|| serde::de::Error::missing_field("cidr"))?;
                let cidr = cidr.parse::<IpCidr>().map_err(serde::de::Error::custom)?;
                let entries = seq
                    .next_element::<Vec<CidrRecord>>()?
                    .ok_or_else(|| serde::de::Error::missing_field("subnets"))?;

                load_cidrs(&entries, &cidr).map_err(serde::de::Error::custom)
            }
            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'s>,
            {
                let mut cidr: Option<IpCidr> = None;
                let mut entries: Option<Vec<CidrRecord>> = None;
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
                load_cidrs(&subnets, &cidr).map_err(serde::de::Error::custom)
            }
        }
        const FIELDS: &[&str] = &["cidr", "subnets"];
        deserializer.deserialize_struct("SubnetPool", FIELDS, SubnetPoolVisitor)
    }
}
