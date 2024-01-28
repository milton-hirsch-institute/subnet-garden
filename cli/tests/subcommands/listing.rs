// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::fixture;
use crate::fixture::Test;
mod cidrs {
    use super::*;
    fn new_cidrs_test() -> Test {
        let mut test = fixture::new_test();
        test.store();
        test.subg.arg("cidrs");
        test
    }

    #[test]
    fn no_cidrs() {
        let mut test = new_cidrs_test();
        test.subg.assert().success().stdout("").stderr("");
    }

    #[test]
    fn has_cidrs() {
        let mut test = new_cidrs_test();
        test.pool.allocate(4, Some("test1")).unwrap();
        test.pool.allocate(6, Some("test2")).unwrap();
        test.store();
        test.subg
            .assert()
            .success()
            .stdout("10.10.0.0/28\n10.10.0.64/26\n")
            .stderr("");
    }

    #[test]
    fn has_cidrs_long() {
        let mut test = new_cidrs_test();
        test.subg.arg("-l");
        test.pool.allocate(4, Some("test1")).unwrap();
        test.pool.allocate(6, None).unwrap();
        test.pool.allocate(6, Some("test2")).unwrap();
        test.store();
        test.subg
            .assert()
            .success()
            .stdout(
                "total 3\n\
                 10.10.0.0/28    test1\n\
                 10.10.0.64/26   -\n\
                 10.10.0.128/26  test2\n",
            )
            .stderr("");
    }

    #[test]
    fn subset_cidrs() {
        let mut test = new_cidrs_test();
        test.pool.allocate(4, Some("test1")).unwrap();
        test.pool.allocate(4, Some("test2")).unwrap();
        test.pool.allocate(4, Some("test3")).unwrap();
        test.subg.arg("--within").arg("10.10.0.0/27");
        test.store();
        test.subg
            .assert()
            .success()
            .stdout("10.10.0.0/28\n10.10.0.16/28\n")
            .stderr("");
    }
}

mod names {
    use super::*;
    fn new_names_test() -> Test {
        let mut test = fixture::new_test();
        test.store();
        test.subg.arg("names");
        test
    }

    #[test]
    fn no_names() {
        let mut test = new_names_test();
        test.subg.assert().success().stdout("").stderr("");
    }

    #[test]
    fn has_names() {
        let mut test = new_names_test();
        test.pool.allocate(4, Some("test1")).unwrap();
        test.pool.allocate(5, None).unwrap();
        test.pool.allocate(6, Some("test2")).unwrap();
        test.pool.allocate(4, Some("test0")).unwrap();
        test.store();
        test.subg
            .assert()
            .success()
            .stdout("test0\ntest1\ntest2\n")
            .stderr("");
    }

    #[test]
    fn has_names_long() {
        let mut test = new_names_test();
        test.subg.arg("-l");
        test.pool.allocate(4, Some("test1")).unwrap();
        test.pool.allocate(5, None).unwrap();
        test.pool.allocate(6, Some("test2")).unwrap();
        test.pool.allocate(4, Some("test-zero")).unwrap();
        test.store();
        test.subg
            .assert()
            .success()
            .stdout(
                "total 3 of 4\n\
                 test-zero  10.10.0.16/28\n\
                 test1      10.10.0.0/28\n\
                 test2      10.10.0.64/26\n",
            )
            .stderr("");
    }
}
