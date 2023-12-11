// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::model::SubnetGarden;
use cidr::{IpCidr, Ipv4Cidr, Ipv6Cidr};
use std::net::{Ipv4Addr, Ipv6Addr};

static TEST_CIDR4: IpCidr = IpCidr::V4(match Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 16) {
    Ok(cidr) => cidr,
    _ => panic!("Failed to create test v4 cidr"),
});

static TEST_CIDR6: IpCidr = IpCidr::V6(
    match Ipv6Cidr::new(Ipv6Addr::new(1, 2, 3, 4, 10, 20, 0, 0), 112) {
        Ok(cidr) => cidr,
        _ => panic!("Failed to create test v6 cidr"),
    },
);

fn new_test_space() -> Memory {
    let mut instance = Memory::new();
    instance.new_space("test4", TEST_CIDR4).unwrap();
    instance.new_space("test6", TEST_CIDR6).unwrap();
    return instance;
}

mod memory_garde {
    use super::*;

    mod new {
        use super::*;
        #[test]
        fn new_memory_garden() {
            let instance = Memory::new();
            assert_eq!(instance.space_count(), 0);
        }
    }

    mod new_space {
        use super::*;

        #[test]
        fn duplicate_object() {
            let mut instance = new_test_space();
            let result = instance.new_space("test4", TEST_CIDR4);
            assert_eq!(result.err(), Some(CreateError::DuplicateObject));
        }

        #[test]
        fn success() {
            let instance = new_test_space();
            assert_eq!(instance.space_count(), 2);
        }
    }

    mod remove_space {
        use super::*;

        #[test]
        fn remove_space_no_such_object() {
            let mut instance = new_test_space();
            let result = instance.remove_space("does-not-exist");
            assert_eq!(result.err(), Some(RemoveError::NoSuchObject));
        }

        #[test]
        fn remove_space_success() {
            let mut instance = new_test_space();
            instance.remove_space("test4").unwrap();
            assert_eq!(instance.space_count(), 1);
        }
    }

    mod space_mut {
        use super::*;

        #[test]
        fn space_success() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            assert_eq!(*space.cidr(), TEST_CIDR4);
        }

        #[test]
        fn space_not_found() {
            let mut instance = new_test_space();
            assert!(instance.space_mut("does-not-exist").is_none());
        }
    }
}

mod space {
    use super::*;

    mod allocate {
        use super::*;

        #[test]
        fn too_many_bits() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let result = space.allocate(17);
            assert_eq!(result.err(), Some(AllocateError::NoSpaceAvailable));
        }

        #[test]
        fn no_space_available() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            space.allocate(16).unwrap();
            let result = space.allocate(16);
            assert_eq!(result.err(), Some(AllocateError::NoSpaceAvailable));
        }

        #[test]
        fn allocate_success_v4() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let result = space.allocate(4).unwrap();
            assert_eq!(
                result,
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap())
            );
        }

        #[test]
        fn allocate_success_v6() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test6").unwrap();
            let result = space.allocate(4).unwrap();
            assert_eq!(
                result,
                &IpCidr::V6(Ipv6Cidr::new(Ipv6Addr::new(1, 2, 3, 4, 10, 20, 0, 0), 124).unwrap())
            );
        }

        #[test]
        fn allocate_multi_sizes() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let result1 = space.allocate(4).unwrap();
            assert_eq!(
                result1,
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap())
            );
            let result2 = space.allocate(8).unwrap();
            assert_eq!(
                result2,
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 1, 0), 24).unwrap())
            );
        }

        #[test]
        fn max_bits() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let result = space.allocate(16).unwrap();
            assert_eq!(
                result,
                &IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 16).unwrap())
            );
        }
    }

    mod list_cidrs {
        use super::*;
        #[test]
        fn no_cidrs() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let cidrs = space.list_cidrs();
            assert_eq!(cidrs.len(), 0);
        }
        #[test]
        fn some() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            space.allocate(4).unwrap();
            space.allocate(5).unwrap();
            space.allocate(5).unwrap();
            space.allocate(4).unwrap();
            space.allocate(4).unwrap();
            space.allocate(4).unwrap();
            let cidrs = space.list_cidrs();
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

    mod claim {
        use super::*;

        #[test]
        fn out_of_range() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 21, 0, 0), 28).unwrap());
            let result = space.claim(&cidr);
            assert_eq!(result, Err(AllocateError::NoSpaceAvailable));
        }

        #[test]
        fn already_claimed() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
            space.claim(&cidr).unwrap();
            let result = space.claim(&cidr);
            assert_eq!(result, Err(AllocateError::NoSpaceAvailable));
        }

        #[test]
        fn already_allocated() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
            space.allocate(16).unwrap();
            let result = space.claim(&cidr);
            assert_eq!(result, Err(AllocateError::NoSpaceAvailable));
        }

        #[test]
        fn success() {
            let mut instance = new_test_space();
            let space = instance.space_mut("test4").unwrap();
            let cidr = IpCidr::V4(Ipv4Cidr::new(Ipv4Addr::new(10, 20, 0, 0), 28).unwrap());
            let result = space.claim(&cidr);
            assert_eq!(result, Ok(()));
        }
    }
}
