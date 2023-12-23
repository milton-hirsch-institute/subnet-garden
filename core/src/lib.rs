// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::errors::{AllocateError, CreateError, DeleteError, RenameError};
use cidr::IpCidr;
use serde::de;
use serde::ser::SerializeStruct;
use std::str::FromStr;

pub mod errors;
pub mod memory;
pub type CreateResult<T> = Result<T, CreateError>;

pub type DeleteResult<T> = Result<T, DeleteError>;

pub type AllocateResult<T> = Result<T, AllocateError>;

pub type RenameResult<T> = Result<T, RenameError>;

pub type Bits = u8;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CidrRecord {
    pub cidr: IpCidr,
    pub name: Option<String>,
}

impl CidrRecord {
    pub(crate) fn new(cidr: IpCidr, name: Option<&str>) -> Self {
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

impl<'s> serde::Deserialize<'s> for CidrRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'s>,
    {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Cidr,
            Name,
        }
        struct CidrRecordVisitor;
        impl<'d> de::Visitor<'d> for CidrRecordVisitor {
            type Value = CidrRecord;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct CidrRecord")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'d>,
            {
                let cidr_str = seq
                    .next_element::<&str>()?
                    .ok_or_else(|| serde::de::Error::missing_field("cidr"))?;
                let cidr = match IpCidr::from_str(cidr_str) {
                    Ok(cidr) => cidr,
                    Err(err) => return Err(serde::de::Error::custom(err)),
                };

                let name = seq
                    .next_element::<Option<&str>>()?
                    .ok_or_else(|| serde::de::Error::missing_field("name"))?;

                Ok(CidrRecord::new(cidr, name))
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'d>,
            {
                let mut cidr: Option<IpCidr> = None;
                let mut name: Option<Option<&str>> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Cidr => {
                            if cidr.is_some() {
                                return Err(serde::de::Error::duplicate_field("cidr"));
                            }
                            cidr = match map.next_value::<&str>() {
                                Ok(cidr) => match IpCidr::from_str(cidr) {
                                    Ok(cidr) => Some(cidr),
                                    Err(err) => return Err(serde::de::Error::custom(err)),
                                },
                                Err(err) => return Err(err),
                            };
                        }
                        Field::Name => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                    }
                }
                let cidr = match cidr {
                    Some(cidr) => cidr,
                    None => return Err(serde::de::Error::missing_field("cidr")),
                };
                let name = match name {
                    Some(name) => name,
                    None => return Err(serde::de::Error::missing_field("name")),
                };
                Ok(CidrRecord::new(cidr, name))
            }
        }

        deserializer.deserialize_struct("CidrRecord", &["cidr", "name"], CidrRecordVisitor)
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

    fn delete_space(&mut self, name: &str) -> DeleteResult<()>;

    fn space_mut(&mut self, name: &str) -> Option<&mut dyn Space>;

    fn space_names(&self) -> Vec<String>;

    fn spaces(&self) -> Vec<&dyn Space>;

    fn entries(&self) -> Vec<(String, &dyn Space)>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use cidr::{Ipv4Cidr, Ipv6Cidr};
    use std::net::{Ipv4Addr, Ipv6Addr};

    pub static TEST_CIDR4: IpCidr =
        IpCidr::V4(match Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 16) {
            Ok(cidr) => cidr,
            _ => panic!("Failed to create test v4 cidr"),
        });

    pub static TEST_CIDR6: IpCidr = IpCidr::V6(
        match Ipv6Cidr::new(Ipv6Addr::new(1, 2, 3, 4, 10, 20, 0, 0), 112) {
            Ok(cidr) => cidr,
            _ => panic!("Failed to create test v6 cidr"),
        },
    );
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
            let record = CidrRecord::new(cidr, name);
            let serialized = serde_json::to_string(&record).unwrap();
            assert_eq!(serialized, r#"{"cidr":"10.20.30.0/24","name":"foo"}"#);
            let unserialized: CidrRecord = serde_json::from_str(&serialized).unwrap();
            assert_eq!(unserialized, record);
        }

        #[test]
        fn deserialize_as_sequence() {
            let cidr = IpCidr::from_str("10.20.30.0/24").unwrap();
            let name = Some("foo");
            let record = CidrRecord::new(cidr, name);
            let serialized = postcard::to_vec::<CidrRecord, 1000>(&record).unwrap();
            let unserialized: CidrRecord = postcard::from_bytes(&serialized).unwrap();
            assert_eq!(unserialized, record);
        }

        #[test]
        fn deserialize_missing_name() {
            let cidr = IpCidr::from_str("10.20.30.0/24").unwrap();
            let record = CidrRecord::new(cidr, None);
            let serialized = serde_json::to_string(&record).unwrap();
            assert_eq!(serialized, r#"{"cidr":"10.20.30.0/24","name":null}"#);
            let unserialized: CidrRecord = serde_json::from_str(&serialized).unwrap();
            assert_eq!(unserialized, record);
        }
    }
}
