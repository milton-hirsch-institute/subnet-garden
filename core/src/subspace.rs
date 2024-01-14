// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::util::host_length;
use crate::Bits;
use crate::{util, CidrRecord};
use cidr::IpCidr;
use cidr_utils::separator;
use std::cmp;

#[derive(Debug, PartialEq)]
pub(crate) enum State {
    Allocated,
    Free,
    Unavailable,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Subspace {
    pub(crate) record: CidrRecord,
    pub(crate) high: Option<Box<Self>>,
    pub(crate) low: Option<Box<Self>>,
    pub(crate) state: State,
    pub(crate) allocated_count: usize,
    pub(crate) max_available_bits: Bits,
}

impl Subspace {
    pub(crate) fn new(cidr: IpCidr) -> Self {
        Subspace {
            record: CidrRecord::new(cidr, None),
            high: None,
            low: None,
            state: State::Free,
            allocated_count: 0,
            max_available_bits: util::host_length(&cidr),
        }
    }

    fn update_info(&mut self) {
        match self.state {
            State::Allocated => {
                self.allocated_count = 1;
                self.max_available_bits = 0;
            }
            State::Free => {
                self.allocated_count = 0;
                self.max_available_bits = host_length(&self.record.cidr);
            }
            State::Unavailable => {
                let low = self.low.as_deref_mut().unwrap();
                let high = self.high.as_deref_mut().unwrap();
                self.allocated_count = low.allocated_count + high.allocated_count;
                self.max_available_bits = cmp::max(low.max_available_bits, high.max_available_bits);
            }
        }
    }

    pub(crate) fn host_length(&self) -> Bits {
        host_length(&self.record.cidr)
    }

    pub(crate) fn split(&mut self) {
        self.state = State::Unavailable;
        let new_network_length = self.record.cidr.network_length() + 1;
        let low_cidr: IpCidr;
        let high_cidr: IpCidr;
        match self.record.cidr {
            IpCidr::V4(cidr) => {
                let subnets = separator::Ipv4CidrSeparator::sub_networks(&cidr, new_network_length);
                let subnets_vec = subnets.unwrap();
                low_cidr = IpCidr::V4(*subnets_vec.get(0).unwrap());
                high_cidr = IpCidr::V4(*subnets_vec.get(1).unwrap());
            }
            IpCidr::V6(cidr) => {
                let subnets = separator::Ipv6CidrSeparator::sub_networks(&cidr, new_network_length);
                let subnets_vec = subnets.unwrap();
                low_cidr = IpCidr::V6(*subnets_vec.get(0).unwrap());
                high_cidr = IpCidr::V6(*subnets_vec.get(1).unwrap());
            }
        }
        self.low = Some(Box::new(Subspace::new(low_cidr)));
        self.high = Some(Box::new(Subspace::new(high_cidr)));
    }

    pub(crate) fn allocate_free_space(
        &mut self,
        host_length: Bits,
        name: Option<&str>,
    ) -> Option<IpCidr> {
        if host_length > self.max_available_bits {
            return None;
        }
        if self.state == State::Free {
            if host_length == self.host_length() {
                self.state = State::Allocated;
                self.update_info();
                self.record.name = name.map(|s| s.to_string());
                return Some(self.record.cidr);
            } else {
                self.split();
            }
        }
        if self.state == State::Unavailable {
            let found_low = self
                .low
                .as_deref_mut()?
                .allocate_free_space(host_length, name);
            return match found_low {
                Some(_) => {
                    self.update_info();
                    found_low
                }
                None => {
                    let found_high = self
                        .high
                        .as_deref_mut()?
                        .allocate_free_space(host_length, name);
                    match found_high {
                        Some(_) => {
                            self.update_info();
                            found_high
                        }
                        None => None,
                    }
                }
            };
        }
        None
    }
    pub(crate) fn free(&mut self, cidr: &IpCidr) -> bool {
        if !util::cidr_contains(&self.record.cidr, cidr) {
            return false;
        }

        match self.state {
            State::Allocated => match self.record.cidr == *cidr {
                true => {
                    self.state = State::Free;
                    self.record.name = None;
                    self.update_info();
                    true
                }
                false => false,
            },
            State::Free => false,
            State::Unavailable => {
                let low = self.low.as_deref_mut().unwrap();
                let high = self.high.as_deref_mut().unwrap();
                let freed = low.free(cidr) || high.free(cidr);
                if freed {
                    if low.state == State::Free && high.state == State::Free {
                        self.low = None;
                        self.high = None;
                        self.state = State::Free;
                    }
                    self.update_info();
                }

                freed
            }
        }
    }

    pub(crate) fn claim(&mut self, cidr: &IpCidr, name: Option<&str>) -> bool {
        if !util::cidr_contains(&self.record.cidr, cidr) {
            return false;
        }

        match self.state {
            State::Allocated => return false,
            State::Free => {
                if self.record.cidr == *cidr {
                    self.state = State::Allocated;
                    self.update_info();
                    self.record.name = name.map(|name| name.to_string());
                    return true;
                }
                self.split();
            }
            State::Unavailable => {}
        }

        if self.low.as_deref_mut().unwrap().claim(cidr, name)
            || self.high.as_deref_mut().unwrap().claim(cidr, name)
        {
            self.update_info();
            return true;
        }
        false
    }

    pub(crate) fn find_record(&self, cidr: &IpCidr) -> Option<&Self> {
        if !util::cidr_contains(&self.record.cidr, cidr) {
            return None;
        }
        if self.record.cidr == *cidr {
            return Some(self);
        }
        let found_low = self.low.as_deref()?.find_record(cidr);
        return match found_low {
            Some(_) => found_low,
            None => self.high.as_deref()?.find_record(cidr),
        };
    }

    pub(crate) fn find_record_mut(&mut self, cidr: &IpCidr) -> Option<&mut Self> {
        if !util::cidr_contains(&self.record.cidr, cidr) {
            return None;
        }
        if self.record.cidr == *cidr {
            return Some(self);
        }
        let found_low = self.low.as_deref_mut()?.find_record_mut(cidr);
        return match found_low {
            Some(_) => found_low,
            None => self.high.as_deref_mut()?.find_record_mut(cidr),
        };
    }
}
