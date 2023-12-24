// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::{AllocateArgs, FreeArgs, SubgArgs};
use cidr::IpCidr;
use std::process::exit;

pub(crate) fn allocate(subg: &SubgArgs, args: &AllocateArgs) {
    let mut garden = crate::load_garden(&subg.garden_path);
    crate::result(
        garden.allocate(args.bits, args.name.as_deref()),
        exitcode::SOFTWARE,
        "Could not allocate subnet",
    );
    crate::store_space(&subg.garden_path, &garden);
}

pub(crate) fn free(subg: &SubgArgs, args: &FreeArgs) {
    let mut garden = crate::load_garden(&subg.garden_path);
    let cidr = match garden.find_by_name(&args.identifier.as_str()) {
        Some(cidr) => cidr,
        None => crate::result(
            args.identifier.parse::<IpCidr>(),
            exitcode::USAGE,
            "Could not parse arg IDENTIFIER",
        ),
    };
    if !garden.free(&cidr) {
        eprintln!("Could not free subnet {}", cidr);
        exit(exitcode::SOFTWARE);
    }
    crate::store_space(&subg.garden_path, &garden);
}

pub(crate) fn cidrs(subg: &SubgArgs) {
    let garden = crate::load_garden(&subg.garden_path);
    for entry in garden.entries() {
        println!("{}", entry.cidr);
    }
}

pub(crate) fn names(subg: &SubgArgs) {
    let garden = crate::load_garden(&subg.garden_path);
    let mut names = garden.names();
    names.sort();
    for name in names {
        println!("{}", name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    mod allocate {
        use super::*;
        fn new_allocate_test(bits: &str, name: Option<&str>) -> Test {
            let mut test = tests::new_test();
            test.store();
            test.subg.arg("allocate").arg(bits);
            if let Some(name) = name {
                test.subg.arg(name);
            }
            test
        }

        #[test]
        fn allocate_failure() {
            let mut test = new_allocate_test("8", Some("test"));
            test.garden.allocate(16, None).unwrap();
            test.store();
            test.subg
                .assert()
                .failure()
                .code(exitcode::SOFTWARE)
                .stdout("")
                .stderr("Could not allocate subnet\nNo space available\n");
        }

        #[test]
        fn allocate_with_name() {
            let mut test = new_allocate_test("8", Some("test"));
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets = test.garden.entries().to_vec();
            assert_eq!(subnets.len(), 1);
            assert_eq!(subnets[0].name.clone().unwrap(), "test");
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/24");
        }

        #[test]
        fn allocate_without_name() {
            let mut test = new_allocate_test("8", None);
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets = test.garden.entries().to_vec();
            assert_eq!(subnets.len(), 1);
            assert_eq!(subnets[0].name, None);
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/24");
        }
    }

    mod free {
        use super::*;
        fn new_free_test(identifier: &str) -> Test {
            let mut test = tests::new_test();
            test.store();
            test.subg.arg("free").arg(identifier);
            test
        }

        #[test]
        fn name_not_found() {
            let mut test = new_free_test("test");
            test.store();
            test.subg
                .assert()
                .failure()
                .code(exitcode::USAGE)
                .stdout("")
                .stderr(
                    "Could not parse arg IDENTIFIER\n\
                couldn\'t parse address in network: invalid IP address syntax\n",
                );
        }

        #[test]
        fn free_failure() {
            let mut test = new_free_test("20.20.0.0/24");
            test.store();
            test.subg
                .assert()
                .failure()
                .code(exitcode::SOFTWARE)
                .stdout("")
                .stderr("Could not free subnet 20.20.0.0/24\n");
        }

        #[test]
        fn free_success_with_name() {
            let mut test = new_free_test("test");
            test.garden.allocate(4, Some("test")).unwrap();
            test.store();
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            assert_eq!(test.garden.find_by_name("test"), None);
        }

        #[test]
        fn free_success_with_cidr() {
            let mut test = new_free_test("10.10.0.0/28");
            test.garden.allocate(4, Some("test")).unwrap();
            test.store();
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            assert_eq!(test.garden.find_by_name("test"), None);
        }
    }

    mod cidrs {
        use super::*;

        fn new_cidrs_test() -> Test {
            let mut test = tests::new_test();
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
            test.garden.allocate(4, Some("test1")).unwrap();
            test.garden.allocate(6, Some("test2")).unwrap();
            test.store();
            test.subg
                .assert()
                .success()
                .stdout("10.10.0.0/28\n10.10.0.64/26\n")
                .stderr("");
        }
    }

    mod names {
        use super::*;

        fn new_names_test() -> Test {
            let mut test = tests::new_test();
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
            test.garden.allocate(4, Some("test1")).unwrap();
            test.garden.allocate(5, None).unwrap();
            test.garden.allocate(6, Some("test2")).unwrap();
            test.garden.allocate(4, Some("test0")).unwrap();
            test.store();
            test.subg
                .assert()
                .success()
                .stdout("test0\ntest1\ntest2\n")
                .stderr("");
        }
    }
}
