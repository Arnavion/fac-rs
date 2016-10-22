An API and manager for Factorio mods. It's a Rust clone of https://github.com/mickael9/fac

The API functionality is split up into separate reusable crates:

- `factorio-mods-api`: API to interface with https://mods.factorio.com/
- `factorio-mods-local`: API to interface with the local Factorio installation.
- `factorio-mods-common`: Common types and functionality used by the other crates.
