// Copyright 2023-2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::tests::*;

use crate::errors::AllocateError;
use cidr::{IpCidr, Ipv4Cidr, Ipv6Cidr};
use itertools::Itertools;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

fn new_test_pool() -> SubnetPool {
    SubnetPool::new(TEST_CIDR4)
}

fn new_test_pool6() -> SubnetPool {
    SubnetPool::new(TEST_CIDR6)
}

mod contains {
    use super::*;

    #[test]
    fn does_not_contain() {
        let mut pool = new_test_pool();
        pool.allocate(4, None).unwrap();
        for cidr in [
            "10.10.0.0/25",
            "10.10.1.0/25",
            "10.10.0.0/23",
            "20.20.0.0/24",
        ] {
            assert!(!pool.contains(&IpCidr::from_str(cidr).unwrap()));
        }
    }
    #[test]
    fn contains() {
        let mut pool = new_test_pool();
        let allocated = pool.allocate(4, None).unwrap();
        assert!(pool.contains(&allocated));
    }
}

mod named_count {
    use super::*;
    #[test]
    fn no_named() {
        let pool = new_test_pool();
        assert_eq!(pool.named_count(), 0);
    }

    #[test]
    fn some_named() {
        let mut pool = new_test_pool();
        pool.allocate(4, Some("a-name")).unwrap();
        pool.allocate(4, Some("b-name")).unwrap();
        pool.allocate(4, None).unwrap();
        assert_eq!(pool.named_count(), 2);
    }
}

mod allocate {
    use super::*;

    #[test]
    fn too_many_bits() {
        let mut pool = new_test_pool();
        let result = pool.allocate(17, None);
        assert_eq!(result.err(), Some(AllocateError::NoSpaceAvailable));
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn no_pool_available() {
        let mut pool = new_test_pool();
        pool.allocate(16, None).unwrap();
        let result = pool.allocate(16, None);
        assert_eq!(result.err(), Some(AllocateError::NoSpaceAvailable));
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn allocate_name_already_exists() {
        let mut pool = new_test_pool();
        pool.allocate(4, Some("a-name")).unwrap();
        let result = pool.allocate(4, Some("a-name"));
        assert_eq!(result.err(), Some(AllocateError::DuplicateName));
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn name_is_not_cidr_record() {
        let mut pool = new_test_pool();
        pool.allocate(4, Some("10.20.0.16/28")).unwrap();
        let result = pool.allocate(4, None);
        assert_eq!(result.err(), None);
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn allocate_named() {
        let mut pool = new_test_pool();
        let result = pool.allocate(4, Some("a-name")).unwrap();
        let looked_up = pool.find_by_name("a-name").unwrap();
        assert_eq!(looked_up, result);
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn allocate_success_v4() {
        let mut pool = new_test_pool();
        let result = pool.allocate(4, None).unwrap();
        assert_eq!(
            result,
            IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap())
        );
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn allocate_success_v6() {
        let mut pool = new_test_pool6();
        let result = pool.allocate(4, None).unwrap();
        assert_eq!(
            result,
            IpCidr::V6(Ipv6Cidr::new(Ipv6Addr::new(1, 2, 3, 4, 10, 20, 0, 0), 124).unwrap())
        );
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn allocate_multi_sizes() {
        let mut pool = new_test_pool();
        let result1 = pool.allocate(4, None).unwrap();
        assert_eq!(
            result1,
            IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap())
        );
        let result2 = pool.allocate(8, None).unwrap();
        assert_eq!(
            result2,
            IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 1, 0), 24).unwrap())
        );
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn max_subnet_allocation() {
        let mut pool = new_test_pool();
        let result = pool.allocate(16, None).unwrap();
        assert_eq!(
            result,
            IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 16).unwrap())
        );
    }

    #[test]
    fn max_available_bits() {
        let mut pool = new_test_pool();
        assert_eq!(pool.max_available_bits(), 16);

        // Use up all the bits!
        let result = pool.allocate(16, None).unwrap();
        assert_eq!(pool.max_available_bits(), 0);
        pool.free(&result);
        assert_eq!(pool.max_available_bits(), 16);

        let net1 = pool.allocate(14, None).unwrap();
        assert_eq!(pool.max_available_bits(), 15);
        let net2 = pool.allocate(15, None).unwrap();
        assert_eq!(pool.max_available_bits(), 14);
        let net3 = pool.allocate(13, None).unwrap();
        assert_eq!(pool.max_available_bits(), 13);
        pool.free(&net1);
        assert_eq!(pool.max_available_bits(), 14);
        pool.free(&net3);
        assert_eq!(pool.max_available_bits(), 15);
        pool.free(&net2);
        assert_eq!(pool.max_available_bits(), 16);
    }
}

mod free {
    use super::*;

    #[test]
    fn out_of_range() {
        let mut pool = new_test_pool();
        assert!(!pool.free(&IpCidr::from_str("20.20.0.0/16").unwrap()));
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn free() {
        let index_list = [0, 1, 2, 3];
        for indices in index_list.iter().permutations(index_list.len()) {
            let mut pool = new_test_pool();
            let mut cidrs = vec![];
            for _ in 0..index_list.len() {
                cidrs.push(pool.allocate(14, None).unwrap());
            }
            fn free_test(pool: &mut SubnetPool, cidr: &IpCidr) {
                assert!(pool.free(cidr));
                assert!(!pool.free(cidr));
                pool.claim(cidr, None).unwrap();
            }
            for index in indices {
                let cidr = cidrs[*index];
                free_test(&mut pool, &cidr);
            }
            assert_eq!(pool.allocated_count(), pool.cidrs().count());
        }
    }
}

mod claim {
    use super::*;
    use crate::errors::AllocateError;

    #[test]
    fn out_of_range() {
        let mut pool = new_test_pool();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 21, 0, 0), 28).unwrap());
        let result = pool.claim(&cidr, None);
        assert_eq!(result, Err(AllocateError::NoSpaceAvailable));
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn already_claimed() {
        let mut pool = new_test_pool();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
        pool.claim(&cidr, None).unwrap();
        let result = pool.claim(&cidr, None);
        assert_eq!(result, Err(AllocateError::NoSpaceAvailable));
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn already_allocated() {
        let mut pool = new_test_pool();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
        pool.allocate(16, None).unwrap();
        let result = pool.claim(&cidr, None);
        assert_eq!(result, Err(AllocateError::NoSpaceAvailable));
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn already_named() {
        let mut pool = new_test_pool();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 16), 28).unwrap());
        pool.allocate(4, Some("a-name")).unwrap();
        let result = pool.claim(&cidr, Some("a-name"));
        assert_eq!(result, Err(AllocateError::DuplicateName));
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn unnamed() {
        let mut pool = new_test_pool();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
        let result = pool.claim(&cidr, None);
        assert_eq!(result, Ok(()));
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }

    #[test]
    fn named() {
        let mut pool = new_test_pool();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
        let result = pool.claim(&cidr, Some("a-name"));
        assert_eq!(result, Ok(()));
        let looked_up = pool.find_by_name("a-name").unwrap();
        assert_eq!(looked_up, cidr);
        assert_eq!(pool.allocated_count(), pool.cidrs().count());
    }
}

mod rename {
    use super::*;
    use crate::errors::RenameError;

    #[test]
    fn not_found() {
        let mut pool = new_test_pool();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 21, 0, 0), 28).unwrap());
        let result = pool.rename(&cidr, Some("a-name"));
        assert_eq!(result, Err(RenameError::NoSuchObject));
        assert_eq!(pool.find_by_name("a-name"), None);
    }
    #[test]
    fn already_not_set() {
        let mut pool = new_test_pool();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
        pool.claim(&cidr, None).unwrap();
        let result = pool.rename(&cidr, None);
        assert_eq!(result, Ok(()));
    }
    #[test]
    fn same_name() {
        let mut pool = new_test_pool();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
        pool.claim(&cidr, Some("same-name")).unwrap();
        let result = pool.rename(&cidr, Some("same-name"));
        assert_eq!(result, Ok(()));
        assert_eq!(pool.find_by_name("same-name").unwrap(), cidr);
    }
    #[test]
    fn already_exists() {
        let mut pool = new_test_pool();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 16), 28).unwrap());
        let existing_cidr = pool.allocate(4, Some("already-exists")).unwrap();
        pool.claim(&cidr, Some("old-name")).unwrap();
        let result = pool.rename(&cidr, Some("already-exists"));
        assert_eq!(result, Err(RenameError::DuplicateName));
        assert_eq!(pool.find_by_name("already-exists").unwrap(), existing_cidr);
        assert_eq!(pool.find_by_name("old-name").unwrap(), cidr);
    }

    #[test]
    fn success() {
        let mut pool = new_test_pool();
        let cidr = pool.allocate(4, Some("old-name")).unwrap();
        let result = pool.rename(&cidr, Some("new-name"));
        assert_eq!(result, Ok(()));
        assert_eq!(pool.find_by_name("new-name").unwrap(), cidr);
        assert_eq!(pool.find_by_name("old-name"), None);
    }
}

mod names {
    use super::*;
    #[test]
    fn success() {
        let mut pool = new_test_pool();
        pool.allocate(4, Some("a-name")).unwrap();
        pool.allocate(4, Some("b-name")).unwrap();
        pool.allocate(4, None).unwrap();
        let mut names: Vec<String> = pool.names().collect();
        names.sort();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "a-name");
        assert_eq!(names[1], "b-name");
    }
}

mod cidrs {
    use super::*;
    #[test]
    fn no_cidrs() {
        let pool = new_test_pool();
        let cidrs = pool.cidrs();
        assert_eq!(cidrs.count(), 0);
    }

    #[test]
    fn some() {
        let mut pool = new_test_pool();
        for bits in [4, 5, 5, 4, 4, 4].iter() {
            pool.allocate(*bits, None).unwrap();
        }
        let mut cidrs = pool.cidrs();
        assert_eq!(
            &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap()),
            cidrs.next().unwrap(),
        );
        assert_eq!(
            &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 16), 28).unwrap()),
            cidrs.next().unwrap(),
        );
        assert_eq!(
            &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 32), 27).unwrap()),
            cidrs.next().unwrap(),
        );
        assert_eq!(
            &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 64), 27).unwrap()),
            cidrs.next().unwrap(),
        );
        assert_eq!(
            &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 96), 28).unwrap()),
            cidrs.next().unwrap(),
        );
        assert_eq!(
            &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 112), 28).unwrap()),
            cidrs.next().unwrap(),
        );
        assert_eq!(cidrs.count(), 0);
    }
}

mod records {
    use super::*;
    use crate::CidrRecord;

    #[test]
    fn success() {
        let mut pool = new_test_pool();
        pool.allocate(4, Some("a-name")).unwrap();
        pool.allocate(4, Some("b-name")).unwrap();
        pool.allocate(4, None).unwrap();
        let mut entries: Vec<&CidrRecord> = pool.records().collect();
        entries.sort();
        assert_eq!(entries.len(), 3);
        assert_eq!(
            entries[0],
            &CidrRecord::new(
                IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap()),
                Some("a-name")
            )
        );
        assert_eq!(
            entries[1],
            &CidrRecord::new(
                IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 16), 28).unwrap()),
                Some("b-name")
            )
        );
        assert_eq!(
            entries[2],
            &CidrRecord::new(
                IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 32), 28).unwrap()),
                None
            )
        );
    }
}
mod serialize {
    use super::*;
    use serde_test::{assert_de_tokens_error, assert_tokens};

    #[test]
    fn parse_bad_network() {
        assert_de_tokens_error::<SubnetPool>(
            &[
                serde_test::Token::Struct {
                    name: "SubnetPool",
                    len: 2,
                },
                serde_test::Token::Str("cidr"),
                serde_test::Token::Str("invalid"),
                serde_test::Token::StructEnd,
            ],
            "couldn't parse address in network: invalid IP address syntax",
        );
    }

    #[test]
    fn allocation_error() {
        assert_de_tokens_error::<SubnetPool>(
            &[
                serde_test::Token::Struct {
                    name: "SubnetPool",
                    len: 2,
                },
                serde_test::Token::Str("cidr"),
                serde_test::Token::Str("10.10.0.0/24"),
                serde_test::Token::Str("subnets"),
                serde_test::Token::Seq { len: Some(3) },
                serde_test::Token::Struct {
                    name: "CidrRecord",
                    len: 2,
                },
                serde_test::Token::Str("cidr"),
                serde_test::Token::Str("10.20.0.0/24"),
                serde_test::Token::Str("name"),
                serde_test::Token::Some,
                serde_test::Token::Str("a-name"),
                serde_test::Token::StructEnd,
                serde_test::Token::Struct {
                    name: "CidrRecord",
                    len: 2,
                },
                serde_test::Token::Str("cidr"),
                serde_test::Token::Str("10.20.0.16/28"),
                serde_test::Token::Str("name"),
                serde_test::Token::Some,
                serde_test::Token::Str("b-name"),
                serde_test::Token::StructEnd,
                serde_test::Token::Struct {
                    name: "CidrRecord",
                    len: 2,
                },
                serde_test::Token::Str("cidr"),
                serde_test::Token::Str("10.20.0.32/28"),
                serde_test::Token::Str("name"),
                serde_test::Token::None,
                serde_test::Token::StructEnd,
                serde_test::Token::SeqEnd,
                serde_test::Token::StructEnd,
            ],
            "No space available",
        );
    }

    #[test]
    fn success() {
        let mut pool = SubnetPool::new(TEST_CIDR4);
        pool.allocate(4, Some("a-name")).unwrap();
        pool.allocate(4, Some("b-name")).unwrap();
        pool.allocate(4, None).unwrap();

        assert_tokens(
            &pool,
            &[
                serde_test::Token::Struct {
                    name: "SubnetPool",
                    len: 2,
                },
                serde_test::Token::Str("cidr"),
                serde_test::Token::Str("10.20.0.0/16"),
                serde_test::Token::Str("subnets"),
                serde_test::Token::Seq { len: Some(3) },
                serde_test::Token::Struct {
                    name: "CidrRecord",
                    len: 2,
                },
                serde_test::Token::Str("cidr"),
                serde_test::Token::Str("10.20.0.0/28"),
                serde_test::Token::Str("name"),
                serde_test::Token::Some,
                serde_test::Token::Str("a-name"),
                serde_test::Token::StructEnd,
                serde_test::Token::Struct {
                    name: "CidrRecord",
                    len: 2,
                },
                serde_test::Token::Str("cidr"),
                serde_test::Token::Str("10.20.0.16/28"),
                serde_test::Token::Str("name"),
                serde_test::Token::Some,
                serde_test::Token::Str("b-name"),
                serde_test::Token::StructEnd,
                serde_test::Token::Struct {
                    name: "CidrRecord",
                    len: 2,
                },
                serde_test::Token::Str("cidr"),
                serde_test::Token::Str("10.20.0.32/28"),
                serde_test::Token::Str("name"),
                serde_test::Token::None,
                serde_test::Token::StructEnd,
                serde_test::Token::SeqEnd,
                serde_test::Token::StructEnd,
            ],
        );
    }
}
