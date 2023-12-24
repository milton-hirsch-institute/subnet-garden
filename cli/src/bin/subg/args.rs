// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use subnet_garden_core::Bits;

pub(crate) const DEFAULT_STORAGE_PATH: &str = "subnet-garden.json";

pub(crate) const SUBG_COMMAND: &str = "subg";

pub(crate) mod init {
    #[derive(Debug, clap::Args)]
    /// Initialize the subnet garden file
    pub(crate) struct InitArgs {
        #[arg(short, long, default_value_t)]
        /// Force initialization even if the garden file already exists
        pub(crate) force: bool,

        #[arg()]
        /// Garden subnet CIDR
        pub(crate) cidr: String,
    }
}

#[derive(Debug, clap::Args)]
/// Allocate subnet
pub(crate) struct AllocateArgs {
    #[arg()]
    /// Number of subnet bits
    pub(crate) bits: Bits,

    #[arg()]
    /// Name of the subnet to allocate
    pub(crate) name: Option<String>,
}

#[derive(Debug, clap::Args)]
/// Free subnet
pub(crate) struct FreeArgs {
    #[arg()]
    /// Name or CIDR of a subnet
    pub(crate) identifier: String,
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum SubgCommands {
    Init(init::InitArgs),
    Allocate(AllocateArgs),
    Free(FreeArgs),
}

#[derive(Debug, clap::Args)]
/// Subnet gardener command line interface
pub(crate) struct SubgArgs {
    #[arg(short, long, default_value = DEFAULT_STORAGE_PATH)]
    pub(crate) garden_path: String,
}

#[derive(Debug, clap::Parser)]
#[command(
    name = SUBG_COMMAND,
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
)]
pub(crate) struct Subg {
    #[command(flatten)]
    pub(crate) args: SubgArgs,

    #[command(subcommand)]
    pub(crate) command: SubgCommands,
}
