# Cargo mirrorer

Tool for dowloading crates and serving them, creating a local alternative for crates.io. Usefull in offline and bad connection situations.

# This repo is under construction TODOs:

## Updating strategy:

- [ ] Offline (only serving crates)
- [ ] One fetch only
- [ ] Scheduled updates
- [ ] Update on network connection available + delay

## Crates fetching:

- [x] All crates (200GB and growing)
- [ ] Top N downloaded crates and their deps
- [ ] Use deps of rust playground
- [ ] Comma separated list of crates
- [ ] Crates from specified Cargo.toml file

## Replicating crates io api

- [x] Git - from full index
- [ ] Git - lean index
- [ ] Sparse index
- [ ] Serving crates
