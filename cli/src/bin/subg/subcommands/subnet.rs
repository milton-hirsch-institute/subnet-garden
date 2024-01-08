// Copyright 2023-2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::{AllocateArgs, CidrsArgs, ClaimArgs, FreeArgs, NamesArgs, RenameArgs, SubgArgs};
use crate::util;
use cidr::IpCidr;
use std::process::exit;

pub(crate) fn allocate(subg: &SubgArgs, args: &AllocateArgs) {
    let mut pool = crate::load_pool(&subg.pool_path);
    crate::result(
        pool.allocate(args.bits, args.name.as_deref()),
        exitcode::SOFTWARE,
        "Could not allocate subnet",
    );
    crate::store_pool(&subg.pool_path, &pool);
}

pub(crate) fn free(subg: &SubgArgs, args: &FreeArgs) {
    let mut pool = crate::load_pool(&subg.pool_path);
    let cidr = match pool.find_by_name(&args.identifier.as_str()) {
        Some(cidr) => cidr,
        None => crate::result(
            args.identifier.parse::<IpCidr>(),
            exitcode::USAGE,
            "Could not parse arg IDENTIFIER",
        ),
    };
    if !pool.free(&cidr) {
        eprintln!("Could not free subnet {}", cidr);
        exit(exitcode::SOFTWARE);
    }
    crate::store_pool(&subg.pool_path, &pool);
}

pub(crate) fn cidrs(subg: &SubgArgs, args: &CidrsArgs) {
    let pool = crate::load_pool(&subg.pool_path);

    if args.long {
        println!("total {}", pool.allocated_count());
    }

    let max_cidr_width = match args.long {
        true => pool
            .records()
            .map(|r| r.cidr.to_string().len())
            .max()
            .unwrap_or(0),
        false => 0,
    };
    for entry in pool.records() {
        let mut cidr = entry.cidr.to_string();
        if args.long {
            util::right_pad(&mut cidr, max_cidr_width);
            let name = entry.name.clone().unwrap_or("-".to_string());
            println!("{}  {}", cidr, name);
        } else {
            println!("{}", cidr);
        }
    }
}

pub(crate) fn names(subg: &SubgArgs, args: &NamesArgs) {
    let pool = crate::load_pool(&subg.pool_path);

    if args.long {
        println!("total {} of {}", pool.named_count(), pool.allocated_count());
    }

    let max_name_width = match args.long {
        true => pool.names().map(|n| n.len()).max().unwrap_or(0),
        false => 0,
    };

    let mut names: Vec<String> = pool.names().collect();
    names.sort();
    for mut name in names {
        if args.long {
            let cidr = pool.find_by_name(&name).unwrap();
            let cidr_string = cidr.to_string();
            util::right_pad(&mut name, max_name_width);
            println!("{}  {}", name, cidr_string);
        } else {
            println!("{}", name);
        }
    }
}

pub(crate) fn claim(subg: &SubgArgs, args: &ClaimArgs) {
    let mut pool = crate::load_pool(&subg.pool_path);
    crate::result(
        pool.claim(&args.cidr, args.name.as_deref()),
        exitcode::SOFTWARE,
        "Could not claim subnet",
    );
    crate::store_pool(&subg.pool_path, &pool);
}

pub(crate) fn rename(subg: &SubgArgs, args: &RenameArgs) {
    let mut pool = crate::load_pool(&subg.pool_path);
    let cidr = match pool.find_by_name(&args.identifier.as_str()) {
        Some(cidr) => cidr,
        None => crate::result(
            args.identifier.parse::<IpCidr>(),
            exitcode::USAGE,
            "Could not parse arg IDENTIFIER",
        ),
    };
    crate::result(
        pool.rename(&cidr, args.name.as_deref()),
        exitcode::SOFTWARE,
        "Could not rename subnet",
    );
    crate::store_pool(&subg.pool_path, &pool);
}

pub(crate) fn max_bits(subg: &SubgArgs) {
    let pool = crate::load_pool(&subg.pool_path);
    let largest = pool.max_available_bits();
    println!("{}", largest);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    mod allocate {
        use super::*;
        use subnet_garden_core::CidrRecord;
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
            test.pool.allocate(16, None).unwrap();
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
            let subnets: Vec<&CidrRecord> = test.pool.records().collect();
            assert_eq!(subnets.len(), 1);
            assert_eq!(subnets[0].name.clone().unwrap(), "test");
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/24");
        }

        #[test]
        fn allocate_without_name() {
            let mut test = new_allocate_test("8", None);
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets: Vec<&CidrRecord> = test.pool.records().collect();
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
            test.pool.allocate(4, Some("test")).unwrap();
            test.store();
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            assert_eq!(test.pool.find_by_name("test"), None);
        }

        #[test]
        fn free_success_with_cidr() {
            let mut test = new_free_test("10.10.0.0/28");
            test.pool.allocate(4, Some("test")).unwrap();
            test.store();
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            assert_eq!(test.pool.find_by_name("test"), None);
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

    mod claim {
        use super::*;
        use subnet_garden_core::CidrRecord;
        fn new_claim_test(cidr: &str, name: Option<&str>) -> Test {
            let mut test = tests::new_test();
            test.store();
            test.subg.arg("claim").arg(cidr);
            if let Some(name) = name {
                test.subg.arg(name);
            }
            test
        }

        #[test]
        fn claim_failed() {
            let mut test = new_claim_test("20.20.0.0/24", Some("does-not-exist"));
            test.subg
                .assert()
                .failure()
                .code(exitcode::SOFTWARE)
                .stdout("")
                .stderr(
                    "Could not claim subnet\n\
                No space available\n",
                );
        }

        #[test]
        fn unnamed() {
            let mut test = new_claim_test("10.10.0.0/24", None);
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets: Vec<&CidrRecord> = test.pool.records().collect();
            assert_eq!(subnets.len(), 1);
            assert_eq!(subnets[0].name, None);
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/24");
        }

        #[test]
        fn named() {
            let mut test = new_claim_test("10.10.0.0/24", Some("test"));
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets: Vec<&CidrRecord> = test.pool.records().collect();
            assert_eq!(subnets.len(), 1);
            assert_eq!(subnets[0].name.clone().unwrap(), "test");
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/24");
        }
    }

    mod rename {
        use super::*;
        use subnet_garden_core::CidrRecord;
        fn new_rename_test(identifier: &str, name: Option<&str>) -> Test {
            let mut test = tests::new_test();
            test.store();
            test.subg.arg("rename").arg(identifier);
            if let Some(name) = name {
                test.subg.arg(name);
            }
            test
        }

        #[test]
        fn unknown() {
            let mut test = new_rename_test("bad-cidr", None);
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
        fn rename_failure() {
            let mut test = new_rename_test("10.10.0.0/24", Some("test"));
            test.pool.allocate(4, Some("test")).unwrap();
            test.pool.allocate(4, None).unwrap();
            test.store();
            test.subg
                .assert()
                .failure()
                .code(exitcode::SOFTWARE)
                .stdout("")
                .stderr("Could not rename subnet\nDuplicate name\n");
        }

        #[test]
        fn success_with_name() {
            let mut test = new_rename_test("test", Some("test2"));
            test.pool.allocate(4, Some("test")).unwrap();
            test.store();
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets: Vec<&CidrRecord> = test.pool.records().collect();
            assert_eq!(subnets.len(), 1);
            assert_eq!(subnets[0].name.clone().unwrap(), "test2");
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/28");
        }

        #[test]
        fn success_with_cidr() {
            let mut test = new_rename_test("10.10.0.0/28", Some("test2"));
            test.pool.allocate(4, Some("test")).unwrap();
            test.store();
            test.subg.assert().success().stdout("").stderr("");
            test.load();
            let subnets: Vec<&CidrRecord> = test.pool.records().collect();
            assert_eq!(subnets.len(), 1);
            assert_eq!(subnets[0].name.clone().unwrap(), "test2");
            assert_eq!(subnets[0].cidr.to_string(), "10.10.0.0/28");
        }
    }

    mod max_available {
        use super::*;
        fn new_max_available_test() -> Test {
            let mut test = tests::new_test();
            test.store();
            test.subg.arg("max-available");
            test
        }

        #[test]
        fn no_subnets() {
            let mut test = new_max_available_test();
            test.subg.assert().success().stdout("16\n").stderr("");
        }

        #[test]
        fn has_subnets() {
            let mut test = new_max_available_test();
            test.pool.allocate(4, Some("test1")).unwrap();
            test.pool.allocate(6, Some("test2")).unwrap();
            test.store();
            test.subg.assert().success().stdout("15\n").stderr("");

            test.pool.allocate(14, Some("test3")).unwrap();
            test.pool.allocate(15, Some("test4")).unwrap();
            test.store();
            test.subg.assert().success().stdout("13\n").stderr("");
        }
    }
}
