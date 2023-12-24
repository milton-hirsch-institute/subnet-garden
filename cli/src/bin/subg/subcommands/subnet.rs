// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::{AllocateArgs, SubgArgs};

pub(crate) fn allocate(subg: &SubgArgs, args: &AllocateArgs) {
    let mut garden = crate::load_garden(&subg.garden_path);
    crate::result(
        garden.allocate(args.bits, args.name.as_deref()),
        exitcode::SOFTWARE,
        "Could not allocate subnet",
    );
    crate::store_space(&subg.garden_path, &garden);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

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
