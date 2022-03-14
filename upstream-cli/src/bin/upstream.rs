// Copyright © 2022 The Radicle Upstream Contributors
//
// This file is part of radicle-upstream, distributed under the GPLv3
// with Radicle Linking Exception. For full terms see the included
// LICENSE file.

#![warn(
    clippy::all,
    clippy::cargo,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]
#![allow(clippy::multiple_crate_versions)]

use anyhow::Context;
use librad::PeerId;

const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-git",
    env!("GIT_HEAD"),
    ".",
    env!("PROFILE")
);

fn main() {
    let program = <Program as clap::Parser>::parse();
    match program.run() {
        Ok(_) => {},
        Err(err) => {
            if let Some(program_error) = err.root_cause().downcast_ref::<ProgramError>() {
                println!("{}", program_error)
            } else {
                println!("{:?}", err)
            }
            std::process::exit(1)
        },
    }
}

#[derive(Debug, clap::Parser)]
#[clap(
    name = "upstream",
    version = VERSION,
    infer_subcommands = true,
    disable_help_subcommand = true,
    propagate_version = true,
    color = clap::ColorChoice::Never
)]
struct Program {
    #[clap(subcommand)]
    command: Command,
    #[clap(flatten)]
    options: Options,
}

impl Program {
    fn run(self) -> anyhow::Result<()> {
        self.command.run(self.options)
    }
}

#[derive(Debug, clap::Parser)]
struct Options {
    #[clap(long, env, global = true)]
    lnk_home: Option<String>,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Create, update or fetch Upstream patches
    Patch {
        #[clap(subcommand)]
        command: PatchCommand,
    },
}

impl Command {
    fn run(self, options: Options) -> anyhow::Result<()> {
        match self {
            Command::Patch { command: commands } => commands.run(options),
        }
    }
}

#[derive(Debug)]
struct PatchHandle {
    peer_id: PeerId,
    name: String,
}

impl std::str::FromStr for PatchHandle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (peer_id_str, name) = s
            .split_once('/')
            .ok_or_else(|| "missing `/` separator".to_string())?;
        let peer_id = librad::PeerId::from_default_encoding(peer_id_str)
            .map_err(|err| format!("invalid peer ID: {}", err))?;
        Ok(PatchHandle {
            peer_id,
            name: name.to_owned(),
        })
    }
}

#[derive(Debug, clap::Subcommand)]
enum PatchCommand {
    /// Creates a patch from your current branch and publishes it to the Radicle network.
    ///
    /// Opens an editor that allows you to edit the patch message.
    Create {},

    /// Updates a patch to the current branch and publishes it to the Radicle network.
    ///
    /// Updates the patch with the same name as the current branch. Sets the patch head to the
    /// current branch head. Opens an editor that allows you to edit the patch message.
    Update {},

    /// Fetch a patch from a peer and create a tag for the patch in the local repository.
    ///
    /// The tag for a patch has the name `radicle-patch/<PATCH_HANDLE>`.
    Fetch {
        /// Patch to fetch in the format <peer id>/<patch name>
        patch_handle: PatchHandle,
    },
}

impl PatchCommand {
    fn run(self, options: Options) -> anyhow::Result<()> {
        match self {
            PatchCommand::Create {} => create_patch(options),
            PatchCommand::Update {} => update_patch(options),
            PatchCommand::Fetch { patch_handle } => fetch_patch(options, patch_handle),
        }
    }
}

fn create_patch(options: Options) -> anyhow::Result<()> {
    let patch_name = get_current_branch_name().context("failed to get current branch name")?;
    let patch_tag_name = format!("radicle-patch/{}", patch_name);

    let lnk_home_env = options.lnk_home.as_ref().map(|value| ("LNK_HOME", value));

    let git_show = std::process::Command::new("git")
        .arg("show")
        .arg("--quiet")
        .arg("HEAD")
        .arg("--pretty=%s%n%n%b")
        .output()
        .context("Could not get latest commit message")?;

    if !git_show.status.success() {
        anyhow::bail!(ProgramError::new("Failed to get latest commit"));
    }

    let last_commit_message =
        String::from_utf8(git_show.stdout).expect("Could not convert `git show` stdout to string");

    let patch_help_message = "# Please describe your patch.
#
# We have pre-filled the patch title and description with information from the
# latest commit on this branch. You can edit it to your liking. The first line
# is the patch title, followed by an empty newline and an optional patch
# description. The patch description supports markdown.
#
# Any lines starting with '#' will be ignored.";

    let exit_status = std::process::Command::new("git")
        .arg("tag")
        .arg("--annotate")
        .arg("--message")
        .arg(format!("{}{}", last_commit_message, patch_help_message))
        .arg("--edit")
        .arg("--force")
        .arg(&patch_tag_name)
        .status()
        .context("Could not create git tag")?;
    if !exit_status.success() {
        anyhow::bail!(ProgramError::new("Failed to create git tag"));
    }

    let exit_status = std::process::Command::new("git")
        .envs(lnk_home_env)
        .arg("push")
        .arg("--force")
        .arg("rad")
        .arg("tag")
        .arg(patch_tag_name)
        .status()
        .context("failed to spawn command")?;
    if !exit_status.success() {
        anyhow::bail!(ProgramError::new("Failed to push git tag"));
    }
    println!("Created patch {}", patch_name);

    Ok(())
}

fn update_patch(options: Options) -> anyhow::Result<()> {
    let patch_name = get_current_branch_name().context("failed to get current branch name")?;
    let patch_tag_name = format!("radicle-patch/{}", patch_name);

    let lnk_home_env = options.lnk_home.as_ref().map(|value| ("LNK_HOME", value));

    let exit_status = std::process::Command::new("git")
        .arg("tag")
        .arg("--annotate")
        .arg("--force")
        .arg(&patch_tag_name)
        .status()
        .context("failed to spawn command")?;
    if !exit_status.success() {
        anyhow::bail!(ProgramError::new("Failed to create git tag"));
    }

    let exit_status = std::process::Command::new("git")
        .envs(lnk_home_env)
        .arg("push")
        .arg("--force")
        .arg("rad")
        .arg("tag")
        .arg(patch_tag_name)
        .status()
        .context("failed to spawn command")?;
    if !exit_status.success() {
        anyhow::bail!(ProgramError::new("Failed to push git tag"));
    }

    println!("Updated patch {}", patch_name);

    Ok(())
}

fn fetch_patch(options: Options, patch_handle: PatchHandle) -> anyhow::Result<()> {
    let lnk_home_env = options.lnk_home.as_ref().map(|value| ("LNK_HOME", value));

    let remote_patch_ref = format!(
        "remotes/{}/tags/radicle-patch/{}",
        patch_handle.peer_id, patch_handle.name
    );
    let local_patch_ref = format!(
        "tags/radicle-patch/{}/{}",
        patch_handle.peer_id, patch_handle.name
    );
    let exit_status = std::process::Command::new("git")
        .envs(lnk_home_env)
        .arg("fetch")
        .arg("rad")
        .arg("--force")
        .arg(format!("{}:{}", remote_patch_ref, local_patch_ref))
        .status()
        .context("failed to spawn command")?;
    if !exit_status.success() {
        anyhow::bail!(ProgramError::new("Failed to push git tag"));
    }

    Ok(())
}

fn get_current_branch_name() -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .stderr(std::process::Stdio::inherit())
        .output()
        .context("failed to spawn command")?;
    if !output.status.success() {
        anyhow::bail!("Command failed with status {:?}", output.status)
    }

    let branch_name = std::str::from_utf8(&output.stdout)
        .context("invalid UTF-8 output from command")?
        .lines()
        .next()
        .ok_or_else(|| anyhow::anyhow!("empty command output"))?;
    Ok(branch_name.to_string())
}

/// Return a `ProgramError` when you want to show an error message to the user without displaying
/// the chain of causes or a backtrace.
#[derive(Debug)]
struct ProgramError {
    message: String,
}

impl ProgramError {
    fn new(message: &(impl ToOwned<Owned = String> + ?Sized)) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}

impl std::error::Error for ProgramError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for ProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
