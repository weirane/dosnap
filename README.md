# dosnap

Automatically manage btrfs snapshots. Inspired by [snapper][] and lilydjwg's
[btrfs-autosnapshot][].

**DISCLAIMER**: This software is in a very early stage. Use it at your own
discretion. I am not responsible for any data loss caused by dosnap.

[snapper]: https://github.com/openSUSE/snapper/
[btrfs-autosnapshot]: https://gist.github.com/lilydjwg/6c4f38d7eb8befb5099d6759941044e1

## Installation

For Arch Linux users, install the binary along with shell completion and systemd
units from the AUR:
```sh
paru -S dosnap-git
```

Install the binary manually:
```sh
git clone --depth 1 https://github.com/weirane/dosnap
cd dosnap
cargo install --path .
```

## Configuration

The default path of the configuration file is `/etc/dosnap.toml`. It can be
overwritten by the `-c` option.

### Global options

```toml
# The device to snapshot
device = "/dev/sda2"
# Mount options for the device. Generally, it should include "subvol=/" if the
# default subvolume is not the top-level subvolume
mount_option = ["subvol=/"]
# Parent directory for all snapshots (relative to top-level)
snapshot_root = "_snapshots"
```

### Subvolume options

```toml
[[subvolumes]]
# Mount point of this subvolume
filesystem = "/home"
# Relative path of this subvolume to the top-level
path = "@home"

# Limits for auto snapshots. Default to usize::MAX
hourly_limit = 10
daily_limit = 10
weekly_limit = 0
monthly_limit = 10
yearly_limit = 10

# Include this filesystem in `dosnap create -a`. Defaults to false
create = true
# Include this filesystem in `dosnap autoclean -a`. Defaults to false
autoclean = true
```

## Usage

For every filesystem, dosnap stores their snapshots under
`<snapshot_root>/<escaped_fs_path>/`. The snapshot name is the current datetime
followed by a suffix. Snapshots with the suffix `-auto` are "auto snapshots",
which can be cleaned with the `autoclean` sub-command according to the limits
specified in the config. The concept is similar to [snapper's][tl-limits].

To automatically create and clean auto snapshots, set the subvolume's `create`
and `autoclean` option to `true`, then start `dosnap-create.timer` and
`dosnap-autoclean.timer`. The default is to create snapshots every two hours and
clean every day.
```sh
sudo systemctl enable --now dosnap-create.timer dosnap-autoclean.timer
```

Below are some of the frequently-used commands. For more detail please refer to
the `-h` flag of the sub-commands or shell completion hints.

[tl-limits]: https://wiki.archlinux.org/index.php/snapper#Set_snapshot_limits

### list

List the snapshots created for `/home`.
```sh
sudo dosnap list /home
```

### create

Create a snapshot for `/home` with the default suffix `-manual`.
```sh
sudo dosnap create /home
```

Create an auto snapshot for `/`. (`--auto` is a short hand for `--suffix=-auto`)
```sh
sudo dosnap create --auto /
```

### autoclean

Clean the auto snapshots for `/home` according to the limits.
```sh
sudo dosnap autoclean /home
```

Clean the auto snapshots for all enabled filesystems.
```sh
sudo dosnap autoclean -a
```

There is a dry-run flag `-d`.

## License

GPL v3. See [LICENSE](./LICENSE).
