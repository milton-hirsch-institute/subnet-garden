// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

pub(crate) const DEFAULT_STORAGE_PATH: &str = "subnet-garden.json";

pub(crate) const SUBG_COMMAND: &str = "subg";

pub(crate) mod init {
    #[derive(Debug, clap::Args)]
    /// Initialize the subnet garden file
    pub(crate) struct InitArgs {
        #[arg(short, long, default_value_t)]
        /// Force initialization even if the garden file already exists
        pub(crate) force: bool,
    }
}

pub(crate) mod space {

    #[derive(Debug, clap::Args)]
    /// Manage spaces
    pub(crate) struct SpaceArgs {
        #[command(subcommand)]
        pub(crate) command: SpaceCommands,
    }

    #[derive(Debug, clap::Args)]
    /// Create a new space.
    pub(crate) struct SpaceNewArgs {
        #[arg()]
        /// The name of the space
        pub(crate) name: String,

        #[arg()]
        /// The managed CIDR space
        pub(crate) cidr: String,
    }

    #[derive(Debug, clap::Subcommand)]
    pub(crate) enum SpaceCommands {
        New(SpaceNewArgs),
    }
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum SubgCommands {
    Init(init::InitArgs),
    Space(space::SpaceArgs),
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
