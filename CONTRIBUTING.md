# Contributing

When contributing to this repository, please first discuss the change you wish to make via issue,
email, or any other method with the owners of this repository before making a change. 

Please note we follow the [Rust code of conduct](https://www.rust-lang.org/policies/code-of-conduct).

## Pull Request Process

1. Ensure any install or build dependencies are removed before the end of the layer when doing a 
   build.
2. Update the README.md with details of changes to the interface, this includes new environment 
   variables, exposed ports, useful file locations and container parameters.
3. Increase the version numbers in any examples files and the README.md to the new version that this
   Pull Request would represent. The versioning scheme we use is [SemVer](http://semver.org/).
4. You may merge the Pull Request in once you have the sign-off of two other developers, or if you 
   do not have permission to do that, you may request the second reviewer to merge it for you.

Beyond these notes, new contributors may find it helpful to store these commands in a local shell
function or hotkey. They will help you on your journey.

### Running doctests:

```
cargo test --doc
```

### Running all tests:

```
cargo test
```

### Linting with clippy:

```
cargo clippy --all-targets --no-default-features
```

### Pre-Commit:

```
cargo fmt -- --checK
```
If this does not already work in your development environment, you might need to run
`rustup component add rustfmt` and try again.

