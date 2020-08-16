`fac` is a mod manager for [Factorio.](https://www.factorio.com) It can be used to install and update mods for the game.

`fac` has a few advantages over the game's built-in mod manager:

- Much faster, since it doesn't require starting and restarting the game for every change.

- Automatically downloads required dependencies and keeps them up-to-date. Adds new dependencies and removes unneeded ones.

- Automatically resolves version conflicts using a package solver.

- Can be configured to download specific versions that aren't the latest version, using semantic versioning.

`fac` is inspired by https://github.com/mickael9/fac


# Build

```bash
cargo +nightly build --release
```

This creates the `fac` binary at `./target/release/fac`

`fac` requires a nightly compiler since it uses an unstable feature (impl-trait-type-aliases).


# Run

```bash
# Global help
fac --help


# Installs the mods named "foo" and "bar". Also adds them to the config file.
# If they have any dependencies, those are also installed recursively.
fac install foo bar

# Removes the mods named "foo" and "bar". Also removes them from the config file.
# If they have any dependencies that are no longer necessary, those are also removed recursively.
fac remove foo bar

# Updates all mods to the latest versions. For mods specified in the config file, they are updated to the version specified in the config file.
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

You can maintain multiple config files with arbitrary names and choose which one to use with the `-c` parameter. For example, you might want to have a default `config.json` for one save, and a `config.bobangels.json` for another save. You can then use `fac update` when you want to play the first save, and `fac -c config.bobangels.json update` when you want to play the second save. This is particularly useful for multiplayer games.

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

## Public API crates

- `factorio-mods-local`: API to interface with the local Factorio installation.
- `factorio-mods-web`: API to search mods / download mods / show mod info from https://mods.factorio.com/
- `factorio-mods-common`: Common types and functionality used by the factorio-mods-* crates.

## Internal crates

- `derive-struct`: Custom derives used by the factorio-mods-* crates.
- `package`: Package solver used by fac-rs.

See their individual crate docs for details.


# License

```
fac-rs

https://github.com/Arnavion/fac-rs

Copyright 2016 Arnav Singh

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

Factorio content and materials are trademarks and copyrights of [Wube software.](https://www.factorio.com/terms-of-service)
