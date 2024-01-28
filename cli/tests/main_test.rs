// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

mod fixture;
mod subcommands;

#[test]
fn test_bin() {
    let mut test = fixture::new_test();
    test.subg
        .assert()
        .failure()
        .code(fixture::HELP_EXIT_CODE)
        .stderr(predicates::str::contains(
            "\'subg\' requires a subcommand but one was not provided",
        ));
}
