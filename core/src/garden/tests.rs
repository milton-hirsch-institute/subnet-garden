// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::tests::*;

use crate::errors::AllocateError;
use cidr::{IpCidr, Ipv4Cidr, Ipv6Cidr};
use itertools::Itertools;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

fn new_test_garden() -> SubnetGarden {
    SubnetGarden::new(TEST_CIDR4)
}

fn new_test_garden6() -> SubnetGarden {
    SubnetGarden::new(TEST_CIDR6)
}

mod contains {
    use super::*;

    #[test]
    fn does_not_contain() {
        let mut space = new_test_garden();
        space.allocate(4, None).unwrap();
        for cidr in [
            "10.10.0.0/25",
            "10.10.1.0/25",
            "10.10.0.0/23",
            "20.20.0.0/24",
        ] {
            assert!(!space.contains(&IpCidr::from_str(cidr).unwrap()));
        }
    }
    #[test]
    fn contains() {
        let mut space = new_test_garden();
        let allocated = space.allocate(4, None).unwrap();
        assert!(space.contains(&allocated));
    }
}

mod allocate {
    use super::*;

    #[test]
    fn too_many_bits() {
        let mut space = new_test_garden();
        let result = space.allocate(17, None);
        assert_eq!(result.err(), Some(AllocateError::NoSpaceAvailable));
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn no_space_available() {
        let mut space = new_test_garden();
        space.allocate(16, None).unwrap();
        let result = space.allocate(16, None);
        assert_eq!(result.err(), Some(AllocateError::NoSpaceAvailable));
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn allocate_name_already_exists() {
        let mut space = new_test_garden();
        space.allocate(4, Some("a-name")).unwrap();
        let result = space.allocate(4, Some("a-name"));
        assert_eq!(result.err(), Some(AllocateError::DuplicateName));
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn name_is_not_cidr_record() {
        let mut space = new_test_garden();
        space.allocate(4, Some("10.20.0.16/28")).unwrap();
        let result = space.allocate(4, None);
        assert_eq!(result.err(), None);
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn allocate_named() {
        let mut space = new_test_garden();
        let result = space.allocate(4, Some("a-name")).unwrap();
        let looked_up = space.find_by_name("a-name").unwrap();
        assert_eq!(looked_up, result);
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn allocate_success_v4() {
        let mut space = new_test_garden();
        let result = space.allocate(4, None).unwrap();
        assert_eq!(
            result,
            IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap())
        );
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn allocate_success_v6() {
        let mut space = new_test_garden6();
        let result = space.allocate(4, None).unwrap();
        assert_eq!(
            result,
            IpCidr::V6(Ipv6Cidr::new(Ipv6Addr::new(1, 2, 3, 4, 10, 20, 0, 0), 124).unwrap())
        );
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn allocate_multi_sizes() {
        let mut space = new_test_garden();
        let result1 = space.allocate(4, None).unwrap();
        assert_eq!(
            result1,
            IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap())
        );
        let result2 = space.allocate(8, None).unwrap();
        assert_eq!(
            result2,
            IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 1, 0), 24).unwrap())
        );
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn max_bits() {
        let mut space = new_test_garden();
        let result = space.allocate(16, None).unwrap();
        assert_eq!(
            result,
            IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 16).unwrap())
        );
    }
}

mod free {
    use super::*;

    #[test]
    fn out_of_range() {
        let mut space = new_test_garden();
        assert!(!space.free(&IpCidr::from_str("20.20.0.0/16").unwrap()));
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn free() {
        let index_list = vec![0, 1, 2, 3];
        for indices in index_list.iter().permutations(index_list.len()) {
            let mut space = new_test_garden();
            let mut cidrs = vec![];
            for _ in 0..index_list.len() {
                cidrs.push(space.allocate(14, None).unwrap());
            }
            fn free_test(garden: &mut SubnetGarden, cidr: &IpCidr) {
                assert!(garden.free(&cidr));
                assert!(!garden.free(&cidr));
                garden.claim(cidr, None).unwrap();
            }
            for index in indices {
                let cidr = cidrs[*index];
                free_test(&mut space, &cidr);
            }
            assert_eq!(space.allocated_count(), space.cidrs().len());
        }
    }
}

mod claim {
    use super::*;
    use crate::errors::AllocateError;

    #[test]
    fn out_of_range() {
        let mut space = new_test_garden();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 21, 0, 0), 28).unwrap());
        let result = space.claim(&cidr, None);
        assert_eq!(result, Err(AllocateError::NoSpaceAvailable));
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn already_claimed() {
        let mut space = new_test_garden();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
        space.claim(&cidr, None).unwrap();
        let result = space.claim(&cidr, None);
        assert_eq!(result, Err(AllocateError::NoSpaceAvailable));
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn already_allocated() {
        let mut space = new_test_garden();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
        space.allocate(16, None).unwrap();
        let result = space.claim(&cidr, None);
        assert_eq!(result, Err(AllocateError::NoSpaceAvailable));
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn already_named() {
        let mut space = new_test_garden();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 16), 28).unwrap());
        space.allocate(4, Some("a-name")).unwrap();
        let result = space.claim(&cidr, Some("a-name"));
        assert_eq!(result, Err(AllocateError::DuplicateName));
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn unnamed() {
        let mut space = new_test_garden();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
        let result = space.claim(&cidr, None);
        assert_eq!(result, Ok(()));
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }

    #[test]
    fn named() {
        let mut space = new_test_garden();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
        let result = space.claim(&cidr, Some("a-name"));
        assert_eq!(result, Ok(()));
        let looked_up = space.find_by_name("a-name").unwrap();
        assert_eq!(looked_up, cidr);
        assert_eq!(space.allocated_count(), space.cidrs().len());
    }
}

mod rename {
    use super::*;
    use crate::errors::RenameError;

    #[test]
    fn not_found() {
        let mut space = new_test_garden();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 21, 0, 0), 28).unwrap());
        let result = space.rename(&cidr, Some("a-name"));
        assert_eq!(result, Err(RenameError::NoSuchObject));
        assert_eq!(space.find_by_name("a-name"), None);
    }
    #[test]
    fn already_not_set() {
        let mut space = new_test_garden();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
        space.claim(&cidr, None).unwrap();
        let result = space.rename(&cidr, None);
        assert_eq!(result, Ok(()));
    }
    #[test]
    fn same_name() {
        let mut space = new_test_garden();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
        space.claim(&cidr, Some("same-name")).unwrap();
        let result = space.rename(&cidr, Some("same-name"));
        assert_eq!(result, Ok(()));
        assert_eq!(space.find_by_name("same-name").unwrap(), cidr);
    }
    #[test]
    fn already_exists() {
        let mut space = new_test_garden();
        let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 16), 28).unwrap());
        let existing_cidr = space.allocate(4, Some("already-exists")).unwrap();
        space.claim(&cidr, Some("old-name")).unwrap();
        let result = space.rename(&cidr, Some("already-exists"));
        assert_eq!(result, Err(RenameError::DuplicateName));
        assert_eq!(space.find_by_name("already-exists").unwrap(), existing_cidr);
        assert_eq!(space.find_by_name("old-name").unwrap(), cidr);
    }

    #[test]
    fn success() {
        let mut space = new_test_garden();
        let cidr = space.allocate(4, Some("old-name")).unwrap();
        let result = space.rename(&cidr, Some("new-name"));
        assert_eq!(result, Ok(()));
        assert_eq!(space.find_by_name("new-name").unwrap(), cidr);
        assert_eq!(space.find_by_name("old-name"), None);
    }
}

mod names {
    use super::*;
    #[test]
    fn success() {
        let mut space = new_test_garden();
        space.allocate(4, Some("a-name")).unwrap();
        space.allocate(4, Some("b-name")).unwrap();
        space.allocate(4, None).unwrap();
        let mut names = space.names();
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
        let space = new_test_garden();
        let cidrs = space.cidrs();
        assert_eq!(cidrs.len(), 0);
    }

    #[test]
    fn some() {
        let mut space = new_test_garden();
        space.allocate(4, None).unwrap();
        space.allocate(5, None).unwrap();
        space.allocate(5, None).unwrap();
        space.allocate(4, None).unwrap();
        space.allocate(4, None).unwrap();
        space.allocate(4, None).unwrap();
        let cidrs = space.cidrs();
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
}

mod entries {
    use super::*;
    use crate::CidrRecord;

    #[test]
    fn success() {
        let mut space = new_test_garden();
        space.allocate(4, Some("a-name")).unwrap();
        space.allocate(4, Some("b-name")).unwrap();
        space.allocate(4, None).unwrap();
        let mut entries = space.entries();
        entries.sort();
        assert_eq!(entries.len(), 3);
        assert_eq!(
            entries[0],
            CidrRecord::new(
                IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap()),
                Some("a-name")
            )
        );
        assert_eq!(
            entries[1],
            CidrRecord::new(
                IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 16), 28).unwrap()),
                Some("b-name")
            )
        );
        assert_eq!(
            entries[2],
            CidrRecord::new(
                IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 32), 28).unwrap()),
                None
            )
        );
    }

    mod serialize {
        use super::*;
        use serde_json::to_string;

        #[test]
        fn parse_bad_network() {
            let json = r#"{"cidr":"bad-network", "subnets": []}"#;
            let err = serde_json::from_str::<SubnetGarden>(json).unwrap_err();
            assert_eq!(
                err.to_string(),
                "couldn't parse address in network: \
                    invalid IP address syntax at line 1 column 21"
                    .to_string()
            );
        }

        #[test]
        fn allocation_error() {
            let json =
                r#"{"cidr":"10.10.0.0/24", "subnets": [{"cidr": "10.20.0.0/24", "name": null}]}"#;
            let err = serde_json::from_str::<SubnetGarden>(json).unwrap_err();
            assert_eq!(
                err.to_string(),
                "No space available at line 1 column 76".to_string()
            );
        }

        #[test]
        fn success() {
            let mut space = SubnetGarden::new(TEST_CIDR4);
            space.allocate(4, Some("a-name")).unwrap();
            space.allocate(4, Some("b-name")).unwrap();
            space.allocate(4, None).unwrap();

            let json = to_string(&space).unwrap();
            assert_eq!(
                json,
                "{\
                    \"cidr\":\"10.20.0.0/16\",\
                    \"subnets\":[\
                    {\"cidr\":\"10.20.0.0/28\",\"name\":\"a-name\"},\
                    {\"cidr\":\"10.20.0.16/28\",\"name\":\"b-name\"},\
                    {\"cidr\":\"10.20.0.32/28\",\"name\":null}\
                    ]}"
            );
            let deserialize: SubnetGarden = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialize, space);
        }
    }
}
