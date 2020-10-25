# cargoman

cargoman provides a basic CLI to manipulate `Cargo.toml` files.

At the moment, it can perform two tasks that are frequently necessary for
RPM packaging:

- normalize targets (flatten "targets" by either making the dependencies
target-independent or by removing them)
- override dependency versions

