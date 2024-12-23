# Tuistash

A Terminal User Interface (TUI) for monitoring Logstash 🪵

![demo](docs/img/demo.gif)

## Installation

### Arch Linux
[tuistash](https://aur.archlinux.org/packages/tuistash) is available as a package in the AUR.
You can install it using an AUR helper (e.g. `paru`):
```shell
paru -S tuistash
```

### Homebrew
```shell
brew tap edmocosta/homebrew-tap
```

```shell
brew install tuistash
```

### Manually
Download the latest release from the [GitHub releases page](https://github.com/edmocosta/tuistash/releases) or build it from the source:

1 - Install Rust and Cargo (Linux and macOS):
```shell
curl https://sh.rustup.rs -sSf | sh
```

2 - Clone the repository:
```shell
git clone https://github.com/edmocosta/tuistash.git
```

3 - Build the binary (`target/release/tuistash`)
```shell
cd tuistash
```

```shell
cargo build --release
```

## Usage

The Logstash's [monitoring API](https://www.elastic.co/guide/en/logstash/current/monitoring-logstash.html) must be enabled
and accessible from the client machine, unless the data is being read from a Logstash diagnostic path.

```shell
$ ./tuistash --help
```

```shell
Usage: tuistash [OPTIONS] [COMMAND]

Commands:
  get   Query data from the Logstash API
  tui   Logstash TUI
  help  Print this message or the help of the given subcommand(s)

Options:
      --host <HOST>                        [default: http://localhost:9600]
      --username <USERNAME>                
      --password <PASSWORD>                
      --skip-tls-verification              
  -p, --diagnostic-path <DIAGNOSTIC_PATH>  Read the data from a Logstash diagnostic path
  -h, --help                               Print help
  -V, --version                            Print version

```

### TUI

```shell
./tuistash
```

```shell
./tuistash tui --help
```

```shell
Logstash TUI

Usage: tuistash tui [OPTIONS]

Options:
  -i, --interval <INTERVAL>    Refresh interval in seconds [default: 1]
```

### Other commands

#### GET

```shell
./tuistash get node --help
```

```shell
Prints the current node information

Usage: tuistash get node [OPTIONS] [TYPES]

Arguments:
  [TYPES]  Valid values are 'node', 'os', 'jvm', 'pipelines' separated by comma

Options:
  -o <OUTPUT> Valid values are 'json', 'raw'
```

Examples:

```shell
./tuistash get node pipelines,os
```

```shell
./tuistash get node jvm -o raw
```
