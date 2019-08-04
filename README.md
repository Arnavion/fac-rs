[![Build Status](https://travis-ci.org/Arnavion/fac-rs.svg?branch=master)](https://travis-ci.org/Arnavion/fac-rs)

`fac` is a mod manager for [Factorio.](https://www.factorio.com) It can be used to install and update mods for the game.

`fac` has a few advantages over the game's built-in mod manager:

- Much faster, since it doesn't require starting the game.

- Automatically downloads required dependencies without needing you to list them.

- Automatically resolves version conflicts using a package solver.

- Can be configured to download specific versions that aren't the latest version, using semantic versioning.

`fac` is inspired by https://github.com/mickael9/fac


# Build

```bash
cargo +nightly build --release
```

This creates the `fac` binary at `./target/release/fac`

`fac` requires a nightly compiler since it uses some unstable features (async-await, impl-trait-type-aliases).


# Run

```bash
# Global help
fac --help


# Installs the specified mods and adds them to the config file
fac install ...

# Deletes the specified mods and removes them from the config file
fac remove ...

# Updates all mods with the latest versions as determined by the config file
fac update
```

`fac` uses a config file to determine which mods should be installed. This file is called `config.json` by default, and is stored in `C:\Users\<>\AppData\Local\fac` on Windows and `~/.config/fac` on Linux.

Here is an example config file:

```json
{
  "version": "V1",
  "mods": {
    "AutoDeconstruct": "*",
    "Bottleneck": "*",
    "FNEI": "*",
    "FasterStart": "*",
    "LB-Modular-Chests": "*",
    "LightedPolesPlus": "*",
    "LoaderRedux": "*",
    "SmartTrains": "*",
    "Squeak Through": "*",
    "helmod": "*"
  }
}
```

Each key in the `mods` object is the name of the mod as it appears in the mod URL (eg https://mods.factorio.com/mod/AutoDeconstruct ). Note that the names are case-sensitive. The value is a semantic version range, like `*` (latest version), `0.1` (the latest 0.1.x version), `=0.1.12` (specifically v0.1.12), etc.

If the config file doesn't exist, `fac` will create a default one with all the mods that are already installed in the game directory.

You can maintain multiple config files with arbitrary names and choose which one to use with the `-c` parameter. For example, you might want to have a default `config.json` for one game, and a `config.bobangels.json` for another game. You can then use `fac update` when you want to play the first game, and `fac -c .../fac/config.bobangels.json update` when you want to play the second game. This is particularly useful for multiplayer games.

You can get the mod names and versions from the https://mods.factorio.com website. Alterntively you can use `fac search` and `fac show`:

```bash
# Search inside mod names and descriptions case-insensitively
$ fac search autodeconstruct

Auto Deconstruct
    Name: AutoDeconstruct

    This mod marks drills that have no more resources to mine for deconstruction.


# Show the info and available versions of a specific mod, using its "Name"
$ fac show AutoDeconstruct

Name: AutoDeconstruct
Author: mindmix
Title: Auto Deconstruct
Summary: This mod marks drills that have no more resources to mine for deconstruction.
Game versions: ^0.13, ^0.13.14, ^0.14, ^0.15, ^0.16, ^0.17
Releases:
    Version: 0.1.0 Game version: ^0.13
    Version: 0.1.1 Game version: ^0.13
    Version: 0.1.2 Game version: ^0.13
    Version: 0.1.3 Game version: ^0.13
    Version: 0.1.4 Game version: ^0.13.14
    Version: 0.1.5 Game version: ^0.14
    Version: 0.1.6 Game version: ^0.14
    Version: 0.1.7 Game version: ^0.15
    Version: 0.1.8 Game version: ^0.15
    Version: 0.1.9 Game version: ^0.15
    Version: 0.1.10 Game version: ^0.16
    Version: 0.1.11 Game version: ^0.16
    Version: 0.1.12 Game version: ^0.17
```


# API

- `factorio-mods-local`: API to interface with the local Factorio installation.
- `factorio-mods-web`: API to search mods / download mods / show mod info from https://mods.factorio.com/
- `factorio-mods-common`: Common types and functionality used by the other crates.
- `package`: Package solver.

See their individual crate docs for details.
