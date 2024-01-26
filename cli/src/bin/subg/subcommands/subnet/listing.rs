// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::{CidrsArgs, NamesArgs, SubgArgs};
use crate::util;

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

#[cfg(test)]
mod tests {
    use crate::tests;
    use crate::tests::Test;
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
}
