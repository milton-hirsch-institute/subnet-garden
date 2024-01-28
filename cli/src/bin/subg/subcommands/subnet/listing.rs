// Copyright 2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::{CidrsArgs, NamesArgs, SubgArgs};
use crate::util;

pub(crate) fn cidrs(subg: &SubgArgs, args: &CidrsArgs) {
    let pool = subg::load_pool(&subg.pool_path);

    if args.long {
        println!("total {}", pool.allocated_count());
    }

    let start_cidr = match args.within {
        Some(within) => within,
        None => *pool.cidr(),
    };

    let max_cidr_width = match args.long {
        true => pool
            .records_within(&start_cidr)
            .map(|r| r.cidr.to_string().len())
            .max()
            .unwrap_or(0),
        false => 0,
    };
    for entry in pool.records_within(&start_cidr) {
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
    let pool = subg::load_pool(&subg.pool_path);

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
