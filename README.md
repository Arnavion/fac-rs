An API and manager for Factorio mods. It's a Rust clone of https://github.com/mickael9/fac

The functionality is split up into multiple crates so that others can use just the API if they want:

- `factorio-mods-api`: API to interface with https://mods.factorio.com/
- `factorio-mods-local`: API to interface with the local Factorio installation.
- `fac`: The `fac` binary.
- `factorio-mods-common`: Common types and functionality used by the other three crates.
