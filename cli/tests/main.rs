// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

#[test]
fn test_bin() {
    let mut test = new_test();
    test.subg
        .assert()
        .failure()
        .code(HELP_EXIT_CODE)
        .stderr(predicates::str::contains(
            "\'subg\' requires a subcommand but one was not provided",
        ));
}
