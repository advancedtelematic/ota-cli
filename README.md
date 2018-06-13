# OTA command line interface

This repository is for managing OTA Connect packages and campaigns from the command line.

To get started, run `make` without any arguments to see a list of available options:

```
      help : Print this message and exit
       cli : Build the OTA CLI
    docker : Build the OTA CLI using Docker
     clean : Clean-up all build output
```

## Building the CLI

### Using cargo

The easiest way to install Cargo is by following the instructions at [rustup.rs](https://rustup.rs). Afterwards you can build the CLI with `make cli`, which will place a binary named `campaign` in this project's root directory.

### Using docker

Run `make docker` to build the CLI using Docker (for an x86 linux target by default).

## Usage

Run `./ota` without any arguments to print the following help output:

```
ota-cli 0.1.0

USAGE:
    ota [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l, --log-level <log-level>                (optional) Set the logging level
    -c, --campaigner-url <campaigner-url>      Campaigner server
    -z, --credentials-zip <credentials-zip>    Path to credentials.zip

SUBCOMMANDS:
    create    Create a new campaign
    get       Retrieve campaign information
    launch    Launch a created campaign
    stats     Retrieve stats from a campaign
    cancel    Cancel a launched campaign
```

If you pass a subcommand without any arguments, you will receive additional help output for that command. For example, running `./campaign get` will return:

```
campaign-get
Retrieve campaign information

USAGE:
    campaign get [OPTIONS] --campaign-id <uuid>

FLAGS:
    -h, --help    Prints help information

OPTIONS:
    -i, --campaign-id <uuid>                   The campaign ID
    -l, --log-level <log-level>                (optional) Set the logging level
    -c, --campaigner-url <campaigner-url>      Campaigner server
    -z, --credentials-zip <credentials-zip>    Path to credentials.zip
```

You can then pass the required arguments to run the actual command:

`./ota campaign get --credentials-zip ~/credentials.zip --campaigner-url http://campaigner.gw.staging.internal.atsgarage.com --campaign-id 2473e3fe-26fc-4685-a54e-f42d6136a25b`

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
