# OTA command line interface

This repository is for managing OTA Connect packages and campaigns from the command line.

To get started, run `make` without any arguments to see a list of available options:

```
      help : Print this message and exit
       ota : Build the OTA CLI
    docker : Build the OTA CLI using Docker
     clean : Clean-up all build output
```

## Building the CLI

### Using cargo

The easiest way to install Cargo is by following the instructions at [rustup.rs](https://rustup.rs). Afterwards you can build the CLI with `make ota`, which will place a binary named `ota` in this project's root directory.

### Using docker

Run `make docker` to build the CLI using Docker (for an x86 linux target by default).

## Usage

Run `ota` without any arguments to print the following help output:

```
ota-cli 0.1.0

USAGE:
    ota [OPTIONS] --credentials-zip <path> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l, --log-level <level>         (optional) Set the logging level
    -z, --credentials-zip <path>    Path to credentials.zip

SUBCOMMANDS:
    campaign    Manage OTA campaigns
    package     Manage OTA packages
    help        Prints this message or the help of the given subcommand(s)
```

If you pass a subcommand without any arguments, you will receive additional help output for that command. For example, running `./ota campaign` will return:

```
ota-campaign
Manage OTA campaigns

USAGE:
    ota --credentials-zip <path> campaign --campaigner-url <url> <SUBCOMMAND>

FLAGS:
    -h, --help    Prints help information

OPTIONS:
    -c, --campaigner-url <url>    Campaigner server

SUBCOMMANDS:
    create    Create a new campaign
    get       Retrieve campaign information
    launch    Launch a created campaign
    stats     Retrieve stats from a campaign
    cancel    Cancel a launched campaign
    help      Prints this message or the help of the given subcommand(s)
```

You can then pass the required arguments to run the actual command:

`./ota --credentials-zip ~/credentials.zip campaign --campaigner-url http://campaigner.gw.staging.internal.atsgarage.com get --campaign-id 2473e3fe-26fc-4685-a54e-f42d6136a25b`

which will return something like the following output on success:

```
{
  "createdAt": "2018-06-12T13:36:38Z",
  "groups": [
    "a1a69d9a-6e45-11e8-ab4c-38c9863f9476"
  ],
  "id": "2473e3fe-26fc-4685-a54e-f42d6136a25b",
  "name": "some campaign name",
  "namespace": "default",
  "update": "a87e3fb1-953d-4b0f-b5bd-7453b79eb4aa",
  "updatedAt": "2018-06-12T13:36:38Z"
}
```
