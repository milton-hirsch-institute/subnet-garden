// Copyright 2023-2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod listing;

use crate::args::{AllocateArgs, ClaimArgs, FreeArgs, RenameArgs, SubgArgs};
use crate::param_str;
use cidr::IpCidr;
use std::process::exit;

pub(crate) fn allocate(subg: &SubgArgs, args: &AllocateArgs) {
    let mut pool = subg::load_pool(&subg.pool_path);
    match &args.param {
        None => {
            subg::result(
                pool.allocate(args.bits, args.name_format.as_deref()),
                exitcode::SOFTWARE,
                "Could not allocate subnet",
            );
        }
        Some(params) => {
            let format = args.name_format.as_deref().unwrap();
            let param_strs: param_str::format::Args = params.iter().map(|s| s.as_str()).collect();
            let names = subg::result(
                param_str::format::format_strings(format, &param_strs),
                exitcode::SOFTWARE,
                "Could not format subnet names",
            );
            for name in names {
                subg::result(
                    pool.allocate(args.bits, Some(name.to_string().as_str())),
                    exitcode::SOFTWARE,
                    format!("Could not allocate subnet {}", name).as_str(),
                );
            }
        }
    };
    subg::store_pool(&subg.pool_path, &pool);
}

pub(crate) fn free(subg: &SubgArgs, args: &FreeArgs) {
    let mut pool = subg::load_pool(&subg.pool_path);
    let identifier_list = match args.param {
        None => vec![args.identifier_format.clone()],
        Some(ref params) => {
            let format = args.identifier_format.as_str();
            let param_strs: param_str::format::Args = params.iter().map(|s| s.as_str()).collect();
            subg::result(
                param_str::format::format_strings(format, &param_strs),
                exitcode::SOFTWARE,
                "Could not format subnet names",
            )
        }
    };
    for identifier in identifier_list {
        let cidr = match pool.find_by_name(identifier.as_str()) {
            Some(cidr) => cidr,
            None => {
                let parse_result = identifier.parse::<IpCidr>();
                if args.ignore_missing {
                    continue;
                }
                subg::result(
                    parse_result,
                    exitcode::USAGE,
                    format!("Could not parse arg IDENTIFIER: {}", identifier).as_str(),
                )
            }
        };
        if !pool.free(&cidr) && !args.ignore_missing {
            eprintln!("Could not free subnet {}", cidr);
            exit(exitcode::SOFTWARE);
        }
    }
    subg::store_pool(&subg.pool_path, &pool);
}

pub(crate) fn claim(subg: &SubgArgs, args: &ClaimArgs) {
    let mut pool = subg::load_pool(&subg.pool_path);
    subg::result(
        pool.claim(&args.cidr, args.name.as_deref()),
        exitcode::SOFTWARE,
        "Could not claim subnet",
    );
    subg::store_pool(&subg.pool_path, &pool);
}

pub(crate) fn rename(subg: &SubgArgs, args: &RenameArgs) {
    let mut pool = subg::load_pool(&subg.pool_path);
    let cidr = match pool.find_by_name(args.identifier.as_str()) {
        Some(cidr) => cidr,
        None => subg::result(
            args.identifier.parse::<IpCidr>(),
            exitcode::USAGE,
            "Could not parse arg IDENTIFIER",
        ),
    };
    subg::result(
        pool.rename(&cidr, args.name.as_deref()),
        exitcode::SOFTWARE,
        "Could not rename subnet",
    );
    subg::store_pool(&subg.pool_path, &pool);
}

pub(crate) fn max_bits(subg: &SubgArgs) {
    let pool = subg::load_pool(&subg.pool_path);
    let largest = pool.max_available_bits();
    println!("{}", largest);
}
