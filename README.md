# Cargo mirrorer

Tool for replication of crates and serving them in a way replicating crates io. Usefull in offline and bad connection situations.

## Updating strategy:

- [ ] Offline (only serving crates)
- [ ] One fetch only
- [ ] Scheduled updates
- [ ] Update on network connection available + delay

## Crates backup fetching:

- [ ] All crates (200GB and growing)
- [ ] Top N downloaded crates and their deps
- [ ] Use deps of rust playground
- [ ] Comma separated list of crates
- [ ] Crates from specified Cargo.toml file
