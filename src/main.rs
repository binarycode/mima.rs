use anyhow::Result;
use clap::AppSettings::DeriveDisplayOrder;
use clap::AppSettings::DisableVersionFlag;
use clap::AppSettings::PropagateVersion;
use clap::Clap;
use mima::App;
use std::path::PathBuf;

#[derive(Clap)]
#[clap(author)]
#[clap(global_setting = DeriveDisplayOrder)]
#[clap(global_setting = DisableVersionFlag)]
#[clap(setting = PropagateVersion)]
#[clap(version)]
struct Options {
    #[clap(about = "Path to configuration")]
    #[clap(default_value = "./mima.toml")]
    #[clap(long = "config")]
    #[clap(short)]
    config_path: PathBuf,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Clap)]
enum Command {
    #[clap(about = "List all guests")]
    #[clap(alias = "list")]
    #[clap(alias = "guests")]
    ListGuests,

    #[clap(about = "Show guest details")]
    #[clap(alias = "show")]
    #[clap(alias = "guest")]
    ShowGuestDetails {
        #[clap(about = "Guest ID")]
        guest_id: String,
    },

    #[clap(about = "Initialize guest")]
    #[clap(alias = "init")]
    #[clap(alias = "init-guest")]
    InitializeGuest {
        #[clap(about = "Guest ID")]
        guest_id: String,
    },

    #[clap(about = "Start guest")]
    #[clap(alias = "start")]
    StartGuest {
        #[clap(about = "Guest ID")]
        guest_id: String,

        #[clap(about = "Boot from CD-ROM")]
        #[clap(long = "boot-from-cdrom")]
        boot_from_cdrom: bool,

        #[clap(about = "Insert CD-ROM image from specified path")]
        #[clap(long = "cdrom")]
        cdrom_path: Option<PathBuf>,

        #[clap(about = "Insert floppy image from specified path")]
        #[clap(long = "floppy")]
        floppy_path: Option<PathBuf>,
    },

    #[clap(about = "Stop guest")]
    #[clap(alias = "stop")]
    StopGuest {
        #[clap(about = "Guest ID")]
        guest_id: String,

        #[clap(about = "Seconds to wait for soft shutdown")]
        #[clap(default_value = "60")]
        #[clap(long)]
        wait: u64,

        #[clap(about = "Kill the guest immediately")]
        #[clap(long)]
        force: bool,
    },

    #[clap(about = "Wait until the guest shuts down")]
    #[clap(alias = "wait")]
    WaitForGuestToShutdown {
        #[clap(about = "Guest ID")]
        guest_id: String,
    },

    #[clap(about = "Copy file to guest")]
    #[clap(alias = "copy")]
    #[clap(alias = "upload")]
    CopyFileToGuest {
        #[clap(about = "Guest ID")]
        guest_id: String,

        #[clap(about = "File path")]
        path: PathBuf,

        #[clap(about = "Maximum SSH connection timeout")]
        #[clap(default_value = "100")]
        #[clap(long = "timeout")]
        max_connection_timeout: u64,
    },

    #[clap(about = "Execute script on guest")]
    #[clap(alias = "execute")]
    #[clap(alias = "run")]
    ExecuteScriptOnGuest {
        #[clap(about = "Guest ID")]
        guest_id: String,

        #[clap(about = "Script path")]
        path: PathBuf,

        #[clap(about = "Maximum SSH connection timeout")]
        #[clap(default_value = "100")]
        #[clap(long = "timeout")]
        max_connection_timeout: u64,
    },

    #[clap(about = "List snapshots")]
    ListSnapshots {
        #[clap(about = "Guest ID")]
        guest_id: String,
    },

    #[clap(about = "Create new snapshot")]
    #[clap(alias = "snapshot")]
    CreateSnapshot {
        #[clap(about = "Guest ID")]
        guest_id: String,

        #[clap(about = "Snapshot ID")]
        snapshot_id: String,
    },

    #[clap(about = "Delete snapshot")]
    DeleteSnapshot {
        #[clap(about = "Guest ID")]
        guest_id: String,

        #[clap(about = "Snapshot ID")]
        snapshot_id: String,
    },

    #[clap(about = "Apply snapshot")]
    #[clap(alias = "apply")]
    #[clap(alias = "restore")]
    #[clap(alias = "revert")]
    #[clap(alias = "switch")]
    ApplySnapshot {
        #[clap(about = "Guest ID")]
        guest_id: String,

        #[clap(about = "Snapshot ID")]
        snapshot_id: String,
    },

    #[clap(about = "Check if snapshot exists")]
    CheckSnapshot {
        #[clap(about = "Guest ID")]
        guest_id: String,

        #[clap(about = "Snapshot ID")]
        snapshot_id: String,
    },
}

fn main() -> Result<()> {
    let options = Options::parse();
    let app = App::new(&options.config_path)?;

    match options.command {
        Command::ListGuests => app.list_guests()?,
        Command::ShowGuestDetails { guest_id } => app.show_guest_details(guest_id)?,
        Command::InitializeGuest { guest_id } => app.initialize_guest(guest_id)?,
        Command::StartGuest {
            boot_from_cdrom,
            cdrom_path,
            floppy_path,
            guest_id,
        } => app.start_guest(guest_id, boot_from_cdrom, cdrom_path, floppy_path)?,
        Command::StopGuest {
            guest_id,
            wait,
            force,
        } => app.stop_guest(guest_id, wait, force)?,
        Command::WaitForGuestToShutdown { guest_id } => app.wait_for_guest_to_shutdown(guest_id)?,
        Command::CopyFileToGuest {
            guest_id,
            path,
            max_connection_timeout,
        } => app.copy_file_to_guest(guest_id, path, max_connection_timeout)?,
        Command::ExecuteScriptOnGuest {
            guest_id,
            path,
            max_connection_timeout,
        } => app.execute_script_on_guest(guest_id, path, max_connection_timeout)?,
        Command::ListSnapshots { guest_id } => app.list_snapshots(guest_id)?,
        Command::CreateSnapshot {
            guest_id,
            snapshot_id,
        } => app.create_snapshot(guest_id, snapshot_id)?,
        Command::DeleteSnapshot {
            guest_id,
            snapshot_id,
        } => app.delete_snapshot(guest_id, snapshot_id)?,
        Command::ApplySnapshot {
            guest_id,
            snapshot_id,
        } => app.apply_snapshot(guest_id, snapshot_id)?,
        Command::CheckSnapshot {
            guest_id,
            snapshot_id,
        } => app.check_snapshot(guest_id, snapshot_id)?,
    }

    Ok(())
}
