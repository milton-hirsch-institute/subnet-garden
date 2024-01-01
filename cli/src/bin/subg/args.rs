// Copyright 2023-2024 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

use cidr::IpCidr;
use subnet_garden_core::Bits;

pub(crate) const DEFAULT_STORAGE_PATH: &str = "subnet-garden-pool.yaml";

pub(crate) const SUBG_COMMAND: &str = "subg";

pub(crate) mod init {
    use cidr::IpCidr;

    #[derive(Debug, clap::Args)]
    /// Initialize the subnet garden pool file
    pub(crate) struct InitArgs {
        #[arg(short, long, default_value_t)]
        /// Force initialization even if the pool file already exists
        pub(crate) force: bool,

        #[arg()]
        /// Pool subnet CIDR
        pub(crate) cidr: IpCidr,
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

#[derive(Debug, clap::Args)]
/// List allocate CIDRs
pub(crate) struct CidrsArgs {}

#[derive(Debug, clap::Args)]
/// List named subnets
pub(crate) struct NamesArgs {}

#[derive(Debug, clap::Args)]
/// Claim subnet
pub(crate) struct ClaimArgs {
    #[arg()]
    /// CIDR subnet to claim
    pub(crate) cidr: IpCidr,

    #[arg()]
    /// Name of the subnet to claim
    pub(crate) name: Option<String>,
}

#[derive(Debug, clap::Args)]
/// Rename subnet
pub(crate) struct RenameArgs {
    #[arg()]
    /// Name or CIDR of the subnet to rename
    pub(crate) identifier: String,

    #[arg()]
    /// New name of the subnet or omit to remove the name
    pub(crate) name: Option<String>,
}

#[derive(Debug, clap::Args)]
/// Largest available subnet (by bits)
pub(crate) struct MaxAvailableArgs {}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum SubgCommands {
    Allocate(AllocateArgs),
    Cidrs(CidrsArgs),
    Claim(ClaimArgs),
    Free(FreeArgs),
    Init(init::InitArgs),
    MaxAvailable(MaxAvailableArgs),
    Names(NamesArgs),
    Rename(RenameArgs),
}

#[derive(Debug, clap::Args)]
/// Subnet garden command line interface
pub(crate) struct SubgArgs {
    #[arg(short = 'p', long, default_value = DEFAULT_STORAGE_PATH, env = "SUBG_POOL_PATH")]
    pub(crate) pool_path: String,
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
