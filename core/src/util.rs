// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::Bits;
use cidr::IpCidr;

#[inline(always)]
pub fn max_bits(cidr: &IpCidr) -> Bits {
    match cidr {
        IpCidr::V4(_) => 32,
        IpCidr::V6(_) => 128,
    }
}

#[inline(always)]
pub fn host_length(cidr: &IpCidr) -> Bits {
    max_bits(cidr) - cidr.network_length()
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

    mod available_bits {
        use super::*;

        #[test]
        fn v6() {
            assert_eq!(host_length(&TEST_CIDR6), 16);
        }

        #[test]
        fn v4() {
            assert_eq!(host_length(&TEST_CIDR4), 16);
        }
    }

    mod cidr_contains {
        use super::*;
        use cidr_utils::separator;

        fn get_nth_of(cidr: &IpCidr, bits: Bits, d: usize) -> IpCidr {
            let network_bits = max_bits(cidr) - bits;
            match cidr {
                IpCidr::V4(cidr) => {
                    let networks = separator::Ipv4CidrSeparator::sub_networks(cidr, network_bits);
                    IpCidr::V4(*networks.unwrap().get(d).unwrap())
                }
                IpCidr::V6(cidr) => {
                    let networks = separator::Ipv6CidrSeparator::sub_networks(cidr, network_bits);
                    IpCidr::V6(*networks.unwrap().get(d).unwrap())
                }
            }
        }

        #[test]
        fn contained_in() {
            assert!(!cidr_contains(&get_nth_of(&TEST_CIDR4, 12, 2), &TEST_CIDR4));
            assert!(!cidr_contains(&get_nth_of(&TEST_CIDR6, 12, 2), &TEST_CIDR6));
        }

        #[test]
        fn contained_in_low_edge() {
            assert!(!cidr_contains(&get_nth_of(&TEST_CIDR4, 12, 0), &TEST_CIDR4));
        }

        #[test]
        fn contained_in_high_edge() {
            assert!(!cidr_contains(
                &get_nth_of(&TEST_CIDR4, 12, 15),
                &TEST_CIDR4
            ));
        }

        #[test]
        fn entirely_outside() {
            assert!(!cidr_contains(
                &get_nth_of(&TEST_CIDR4, 12, 15),
                &get_nth_of(&TEST_CIDR4, 12, 0)
            ));
            assert!(!cidr_contains(
                &get_nth_of(&TEST_CIDR6, 12, 15),
                &get_nth_of(&TEST_CIDR6, 12, 0)
            ));
        }

        #[test]
        fn equal() {
            assert!(cidr_contains(&TEST_CIDR6, &TEST_CIDR6));
        }

        #[test]
        fn inner() {
            assert!(cidr_contains(&TEST_CIDR4, &get_nth_of(&TEST_CIDR4, 12, 2)));
        }

        #[test]
        fn low_edge() {
            assert!(cidr_contains(&TEST_CIDR4, &get_nth_of(&TEST_CIDR4, 12, 0)));
        }

        #[test]
        fn high_edge() {
            assert!(cidr_contains(&TEST_CIDR4, &get_nth_of(&TEST_CIDR4, 12, 15)));
        }
    }
}
