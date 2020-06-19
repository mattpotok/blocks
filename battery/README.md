# Battery

Displays battery information: the total charge percentage, the total
remaining capacity percentage, charge status, and time to (dis)charge.

## Setup

Build and install the `battery` block.

```sh
cargo install --path . --root ~/.config/i3blocks/
cp weather.yaml ~/.config/i3blocks/cfg/
```

The `battery` block requires an additional YAML configuration file. Create the
file at `~/.config/i3blocks/cfg/` following the template below.

```yaml
# Configuration YAML for `battery` block
# Required
log_file_path: /absolute/path/to/log/file

# Optional
log_batteries: bool [default = false]
```

## Usage

Configure i3blocks.

```
[battery]
command=~/.config/i3blocks/bin/battery ~/.config/i3blocks/cfg/battery.yaml
interval=15
```
