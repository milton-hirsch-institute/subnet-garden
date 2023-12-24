// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::tests::{TEST_CIDR4, TEST_CIDR6};

pub(crate) fn new_test_garden() -> SubnetGardenLibrary {
    let mut instance = SubnetGardenLibrary::new();
    instance.new_space("test4", TEST_CIDR4).unwrap();
    instance.new_space("test6", TEST_CIDR6).unwrap();
    return instance;
}

mod new {
    use super::*;
    #[test]
    fn new_memory_garden() {
        let instance = SubnetGardenLibrary::new();
        assert_eq!(instance.space_count(), 0);
    }
}

mod new_space {
    use super::*;

    #[test]
    fn duplicate_object() {
        let mut instance = new_test_garden();
        let result = instance.new_space("test4", TEST_CIDR4);
        assert_eq!(result.err(), Some(CreateError::DuplicateObject));
    }

    #[test]
    fn success() {
        let instance = new_test_garden();
        assert_eq!(instance.space_count(), 2);
    }
}

mod delete_space {
    use super::*;

    #[test]
    fn delete_space_no_such_object() {
        let mut instance = new_test_garden();
        let result = instance.delete_space("does-not-exist");
        assert_eq!(result.err(), Some(DeleteError::NoSuchObject));
    }

    #[test]
    fn delete_space_success() {
        let mut instance = new_test_garden();
        instance.delete_space("test4").unwrap();
        assert_eq!(instance.space_count(), 1);
    }
}

mod space_mut {
    use super::*;

    #[test]
    fn space_success() {
        let mut instance = new_test_garden();
        let space = instance.space_mut("test4").unwrap();
        assert_eq!(*space.cidr(), TEST_CIDR4);
    }

    #[test]
    fn space_not_found() {
        let mut instance = new_test_garden();
        assert!(instance.space_mut("does-not-exist").is_none());
    }
}

mod space_names {
    use super::*;

    #[test]
    fn space_names_success() {
        let instance = new_test_garden();
        let mut names = instance.space_names();
        names.sort();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "test4");
        assert_eq!(names[1], "test6");
    }

    #[test]
    fn spaces_success() {
        let instance = new_test_garden();
        let mut spaces = instance.spaces();
        spaces.sort_by(|a, b| a.cidr().cmp(b.cidr()));
        assert_eq!(spaces.len(), 2);
        assert_eq!(*spaces[0].cidr(), TEST_CIDR4);
        assert_eq!(*spaces[1].cidr(), TEST_CIDR6);
    }

    #[test]
    fn entries_success() {
        let instance = new_test_garden();
        let mut entries = instance.entries();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "test4");
        assert_eq!(*entries[0].1.cidr(), TEST_CIDR4);
        assert_eq!(entries[1].0, "test6");
        assert_eq!(*entries[1].1.cidr(), TEST_CIDR6);
    }
}

mod serialize {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn success() {
        let instance = new_test_garden();

        let json = to_string(&instance).unwrap();
        assert_eq!(
            json,
            "{\"spaces\":\
                {\"test4\":{\"cidr\":\"10.20.0.0/16\",\"subnets\":[]},\
                \"test6\":{\"cidr\":\"1:2:3:4:a:14::/112\",\"subnets\":[]}}}"
        );

        let deserialize: SubnetGardenLibrary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialize, instance);
    }
}
