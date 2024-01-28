// Copyright 2023-2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use crate::args::{Subg, SubgCommands};

use clap::Parser;
use subcommands::init;
use subcommands::subnet;
use subcommands::subnet::listing;

mod args;
mod param_str;
mod subcommands;
mod util;

fn main() {
    let subg = Subg::parse();

    match subg.command {
        SubgCommands::Init(args) => {
            init::init(&subg.args, &args);
        }
        SubgCommands::Allocate(args) => {
            subnet::allocate(&subg.args, &args);
        }
        SubgCommands::Free(args) => {
            subnet::free(&subg.args, &args);
        }
        SubgCommands::Cidrs(args) => {
            listing::cidrs(&subg.args, &args);
        }
        SubgCommands::Names(args) => {
            listing::names(&subg.args, &args);
        }
        SubgCommands::Claim(args) => {
            subnet::claim(&subg.args, &args);
        }
        SubgCommands::Rename(args) => {
            subnet::rename(&subg.args, &args);
        }
        SubgCommands::MaxAvailable(_) => {
            subnet::max_bits(&subg.args);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Subg::command().debug_assert();
    }
}
