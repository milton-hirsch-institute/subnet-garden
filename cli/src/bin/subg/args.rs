// Copyright 2023 The Milton Hirsch Institute, B.V.
// SPDX-License-Identifier: Apache-2.0

pub(super) const DEFAULT_STORAGE_PATH: &str = "subnet-garden.json";

pub(super) const SUBG_COMMAND: &str = "subg";

#[derive(Debug, clap::Args)]
/// Initialize the subnet garden file
pub(super) struct InitArgs {
    #[arg(short, long, default_value_t)]
    /// Force initialization even if the garden file already exists
    pub(super) force: bool,
}

#[derive(Debug, clap::Args)]
/// Manage spaces
pub(super) struct SpaceArgs {
    #[command(subcommand)]
    pub(super) command: SpaceCommands,
}

#[derive(Debug, clap::Args)]
/// Create a new space.
pub(super) struct SpaceNewArgs {
    #[arg()]
    /// The name of the space
    pub(super) name: String,

    #[arg()]
    /// The managed CIDR space
    pub(super) cidr: String,
}

#[derive(Debug, clap::Subcommand)]
pub(super) enum SpaceCommands {
    New(SpaceNewArgs),
}

#[derive(Debug, clap::Subcommand)]
pub(super) enum SubgCommands {
    Init(InitArgs),
    Space(SpaceArgs),
}

#[derive(Debug, clap::Args)]
/// Subnet gardener command line interface
pub(super) struct SubgArgs {
    #[arg(short, long, default_value = DEFAULT_STORAGE_PATH)]
    pub(super) garden_path: String,
}

#[derive(Debug, clap::Parser)]
#[command(
    name = SUBG_COMMAND,
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
)]
pub(super) struct Subg {
    #[command(flatten)]
    pub(super) args: SubgArgs,

    #[command(subcommand)]
    pub(super) command: SubgCommands,
}
