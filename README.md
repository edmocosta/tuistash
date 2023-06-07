# Tuistash

A terminal user interface for Logstash.

![demo](docs/img/demo.gif)

## Installation

### Homebrew
```shell
brew tap edmocosta/homebrew-tap
```

```shell
brew install tuistash
```

### Manual
Download the latest release from the [github releases page](https://github.com/edmocosta/tuistash/releases).

## Usage

```shell
$ ./tuistash --help
```

```shell
Usage: tuistash [OPTIONS] <COMMAND>

Commands:
  get   Get data from Logstash
  view  Monitoring TUI
  help  Print this message or the help of the given subcommand(s)

Options:
      --host <HOST>            [default: http://localhost:9600]
      --username <USERNAME>    
      --password <PASSWORD>    
      --skip-tls-verification  
  -h, --help                   Print help
  -V, --version                Print version
```

### Monitoring UI:

```shell
./tuistash view
```

#### Shortcuts:
- `<P>`: Switch to the Pipeline view 
- `<N>`: Switch to the Node view
- `<F>`: When a pipeline is selected, shows its flow charts
- `<Enter>`: When a pipeline's component is selected, it shows it details
- `<Up>`,`<Down>`, `<Left>`, `<Right>`: Navigation
- `<H>`: Open the help panel
- `<Q>`, `<Esc>`: Exit

### GET command:

```shell
./tuistash get node --help
```

```shell
Prints the current node information

Usage: tuistash get node [OPTIONS] [TYPES]

Arguments:
  [TYPES]  Valid values are 'node', 'os', 'jvm', 'pipelines' separated by comma

Options:
  -o <OUTPUT> Valid values are 'default', 'json', 'table', 'raw'
```

Examples:

```shell
./tuistash get node pipelines,os
```

```shell
  PIPELINES                                                                                                                                    
  NAME   WORKERS  BATCH_SIZE  BATCH_DELAY  CONFIG_RELOAD_AUTOMATIC  CONFIG_RELOAD_INTERVAL  DLQ_ENABLED  EPHEMERAL_ID                          
  debug  2        125         50           true                     3000000000              false        454ab3a7-92bb-45c6-bef3-91759e20987d  
  
  OS                                                                                                                                           
  NAME   VERSION           ARCH     AVAILABLE_PROCESSORS                                                                                       
  Linux  5.15.49-linuxkit  aarch64  2    
```

```shell
./tuistash get node jvm -o json
```

```shell
{
  "jvm": {
    "gc_collectors": [
      "G1 Young Generation",
      "G1 Old Generation"
    ],
    "mem": {
      "heap_init_in_bytes": 1073741824,
      "heap_max_in_bytes": 1073741824,
      "non_heap_init_in_bytes": 7667712,
      "non_heap_max_in_bytes": 0
    },
    "pid": 1,
    "start_time_in_millis": 1685344376388,
    "version": "17.0.6",
    "vm_name": "OpenJDK 64-Bit Server VM",
    "vm_vendor": "Eclipse Adoptium",
    "vm_version": "17.0.6"
  }
}
```
