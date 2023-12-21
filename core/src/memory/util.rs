// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::Bits;
use cidr::IpCidr;

pub fn max_bits(cidr: &IpCidr) -> Bits {
    match cidr {
        IpCidr::V4(_) => 32,
        IpCidr::V6(_) => 128,
    }
}

pub fn cidr_contains(outer: &IpCidr, inner: &IpCidr) -> bool {
    let first = inner.first_address();
    let last = inner.last_address();
    outer.contains(&first) && outer.contains(&last)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{TEST_CIDR4, TEST_CIDR6};

    mod max_bits {
        use super::*;

        #[test]
        fn v6() {
            assert_eq!(max_bits(&TEST_CIDR6), 128);
        }

        #[test]
        fn v4() {
            assert_eq!(max_bits(&TEST_CIDR4), 32);
        }
    }
}
