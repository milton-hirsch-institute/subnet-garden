// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::errors::{AllocateError, CreateError, RemoveError, RenameError};
use cidr::IpCidr;
use serde::ser::SerializeStruct;

pub type CreateResult<T> = Result<T, CreateError>;

pub type RemoveResult<T> = Result<T, RemoveError>;

pub type AllocateResult<T> = Result<T, AllocateError>;

pub type RenameResult<T> = Result<T, RenameError>;

pub type Bits = u8;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CidrRecord {
    pub cidr: IpCidr,
    pub name: Option<String>,
}

impl CidrRecord {
    pub fn new(cidr: IpCidr, name: Option<&str>) -> Self {
        CidrRecord {
            cidr,
            name: match name {
                Some(name) => Some(name.to_string()),
                None => None,
            },
        }
    }
}

impl serde::Serialize for CidrRecord {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut structure = serializer.serialize_struct("CidrRecord", 2)?;
        structure.serialize_field("cidr", &self.cidr.to_string())?;
        structure.serialize_field("name", &self.name)?;
        structure.end()
    }
}

pub trait Space {
    fn cidr(&self) -> &IpCidr;

    fn find_by_name(&self, name: &str) -> Option<IpCidr>;

    fn allocate(&mut self, host_length: Bits, name: Option<&str>) -> AllocateResult<IpCidr>;

    fn claim(&mut self, cidr: &IpCidr, name: Option<&str>) -> AllocateResult<()>;

    fn rename(&mut self, cidr: &IpCidr, name: Option<&str>) -> RenameResult<()>;

    fn names(&self) -> Vec<String>;

    fn cidrs(&self) -> Vec<&IpCidr>;

    fn entries(&self) -> Vec<CidrRecord>;
}
pub trait SubnetGarden {
    fn space_count(&self) -> usize;
    fn new_space(&mut self, name: &str, cidr: IpCidr) -> CreateResult<&mut dyn Space>;

    fn remove_space(&mut self, name: &str) -> RemoveResult<()>;

    fn space_mut(&mut self, name: &str) -> Option<&mut dyn Space>;

    fn space_names(&self) -> Vec<String>;

    fn spaces(&self) -> Vec<&dyn Space>;

    fn entries(&self) -> Vec<(String, &dyn Space)>;
}

#[cfg(test)]
mod tests {
    use super::*;
    mod cidr_record {
        use super::*;
        use std::str::FromStr;

        #[test]
        fn new() {
            let cidr = IpCidr::from_str("10.20.30.0/24").unwrap();
            let name = Some("foo");
            let record = super::super::CidrRecord::new(cidr, name);
            assert_eq!(record.cidr, cidr);
            assert_eq!(record.name, Some(name.unwrap().to_string()));
        }

        #[test]
        fn serialize() {
            let cidr = IpCidr::from_str("10.20.30.0/24").unwrap();
            let name = Some("foo");
            let record = super::super::CidrRecord::new(cidr, name);
            let serialized = serde_json::to_string(&record).unwrap();
            assert_eq!(serialized, r#"{"cidr":"10.20.30.0/24","name":"foo"}"#);
        }
    }
}
