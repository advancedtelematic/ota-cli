# OTA command line interface

This tool is for managing OTA Connect devices, groups, packages and campaigns. Running `make` without any arguments will display a list of build options:

```
      help : Print this message and exit
       ota : Build the OTA CLI
    docker : Build the OTA CLI using Docker
     clean : Clean-up all build output
```

## Building the CLI

### Using cargo

The easiest way to install Cargo is by following the instructions at [rustup.rs](https://rustup.rs).

After installing Cargo you can build the CLI with `make ota`. This will place the `ota` binary in `$CARGO_HOME/bin` and can be run with `ota`.

### Using docker

You can also build a binary using Docker (for an linux target by default). Running `make docker` will place a binary named `ota` in this project's root directory.

You can now move this binary into your path or remember to substitute subsequent `ota` commands with `./ota` to run it from this directory.

## Usage

Run `ota` without any arguments to print usage output:

```
ota 0.1.0

USAGE:
    ota <SUBCOMMAND>

OPTIONS:
    -l, --level <level>    Set the logging level
    -h, --help             Prints help information
    -V, --version          Prints version information

SUBCOMMANDS:
    init        Set config values before starting
    campaign    Manage OTA campaigns
    device      Manage OTA devices
    group       Manage device groups
    package     Manage OTA packages
    update      Manage multi-target updates
    help        Prints this message or the help of the given subcommand(s)
```

If you pass a subcommand without any arguments, you will receive additional help output for that command.

### Initialise config values

Before running commands against OTA Connect servers, you must first specify the server endpoints to use with `ota init`:

```
ota-init
Set config values before starting

USAGE:
    ota init --campaigner <url> --credentials <zip> --director <url> --registry <url>

OPTIONS:
    -z, --credentials <zip>    Path to credentials.zip
    -c, --campaigner <url>     Campaigner URL
    -d, --director <url>       Director URL
    -r, --registry <url>       Device Registry URL
    -l, --level <level>        Set the logging level
    -h, --help                 Prints help information
```

For example, to run against our internal staging servers, the command would look like:

```
ota init \
  --credentials ~/credentials.zip \
  --campaigner http://campaigner.gw.staging.internal.atsgarage.com \
  --director http://director.gw.staging.internal.atsgarage.com \
  --registry http://device-registry.gw.staging.internal.atsgarage.com
```

### Create a multi-target update

Before launching a campaign, you must first create a multi-target update. Running `ota update create` will show the following help output:

```
ota-update-create
Create a multi-target update

USAGE:
    ota update create --targets <toml>

OPTIONS:
    -h, --help              Prints help information
    -l, --level <level>     Set the logging level
    -t, --targets <toml>    Update targets file
    -V, --version           Prints version information
```

The only required flag is `--targets <toml>`, which outlines the mapping between an ECU's hardware identifier and the package to update to.

You can take a look at `examples/targets.toml` for an example of the targets file layout.

### Launch a campaign

After creating a multi-target update, you can use the returned UUID as an input to `ota campaign create`:

```
ota-campaign-create
Create a new campaign

USAGE:
    ota campaign create --groups <uuid>... --name <name> --update <uuid>

OPTIONS:
    -u, --update <uuid>       Multi-target update id
    -n, --name <name>         A campaign name
    -g, --groups <uuid>...    Apply the campaign to these groups
    -l, --level <level>       Set the logging level
    -h, --help                Prints help information
    -V, --version             Prints version information
```

This will return a campaign UUID which can then be used to launch the campaign with `ota campaign launch --campaign <uuid>`.
