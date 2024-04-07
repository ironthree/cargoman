# cargoman (DEPRECATED)

**WARNING**: This project has been superseded by [rust2rpm-helper] and is no
longer maintained.

[rust2rpm-helper]: https://src.fedoraproject.org/rpms/rust2rpm-helper

---

cargoman provides a basic CLI to manipulate `Cargo.toml` files.

At the moment, it can perform two tasks that are frequently necessary for
RPM packaging:

- normalize targets (flatten "targets" by either making the dependencies
target-independent or by removing them)
- override dependency versions

The goal of this project is to eventually make it unnecessary to manually
patch `Cargo.toml` files for RPM packages of Rust crates in Fedora and
to automate these modifications as part of the build process instead.

