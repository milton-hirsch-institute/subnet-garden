// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::errors::{AllocateError, CreateError, DeleteError, RenameError};
use cidr::IpCidr;
use serde::de;
use serde::ser::SerializeStruct;
use std::str::FromStr;

pub mod errors;
pub mod pool;
mod subspace;
mod util;

pub type CreateResult<T> = Result<T, CreateError>;

pub type DeleteResult<T> = Result<T, DeleteError>;

pub type AllocateResult<T> = Result<T, AllocateError>;

pub type RenameResult<T> = Result<T, RenameError>;

pub type Bits = u8;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct CidrRecord {
    pub cidr: IpCidr,
    pub name: Option<String>,
}

impl CidrRecord {
    pub(crate) fn new(cidr: IpCidr, name: Option<&str>) -> Self {
        CidrRecord {
            cidr,
            name: name.map(|name| name.to_string()),
        }
    }
}

impl serde::Serialize for CidrRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
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
                let mut name: Option<Option<String>> = None;
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
                Ok(CidrRecord::new(cidr, name.as_deref()))
            }
        }

        deserializer.deserialize_struct("CidrRecord", &["cidr", "name"], CidrRecordVisitor)
    }
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
        use serde_test::{assert_de_tokens_error, assert_tokens};
        use std::str::FromStr;

        #[test]
        fn new() {
            let cidr = IpCidr::from_str("10.20.30.0/24").unwrap();
            let name = Some("foo");
            let record = super::super::CidrRecord::new(cidr, name);
            assert_eq!(record.cidr, cidr);
            assert_eq!(record.name.as_deref(), name);
        }

        #[test]
        fn serialize_named() {
            let cidr = IpCidr::from_str("10.20.30.0/24").unwrap();
            let record = CidrRecord::new(cidr, Some("a-record"));
            assert_tokens(
                &record,
                &[
                    serde_test::Token::Struct {
                        name: "CidrRecord",
                        len: 2,
                    },
                    serde_test::Token::Str("cidr"),
                    serde_test::Token::Str("10.20.30.0/24"),
                    serde_test::Token::Str("name"),
                    serde_test::Token::Some,
                    serde_test::Token::Str("a-record"),
                    serde_test::Token::StructEnd,
                ],
            );
        }

        #[test]
        fn serialize_unnamed() {
            let cidr = IpCidr::from_str("10.20.30.0/24").unwrap();
            let record = CidrRecord::new(cidr, None);
            assert_tokens(
                &record,
                &[
                    serde_test::Token::Struct {
                        name: "CidrRecord",
                        len: 2,
                    },
                    serde_test::Token::Str("cidr"),
                    serde_test::Token::Str("10.20.30.0/24"),
                    serde_test::Token::Str("name"),
                    serde_test::Token::None,
                    serde_test::Token::StructEnd,
                ],
            );
        }

        #[test]
        fn deserialize_as_sequence() {
            assert_de_tokens_error::<CidrRecord>(
                &[
                    serde_test::Token::Struct {
                        name: "CidrRecord",
                        len: 2,
                    },
                    serde_test::Token::Str("cidr"),
                    serde_test::Token::Str("invalid"),
                    serde_test::Token::StructEnd,
                ],
                "couldn't parse address in network: invalid IP address syntax",
            );
        }
    }
}
