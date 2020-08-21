# Transx &middot; [![GitHub license](https://img.shields.io/badge/license-GPL3%2FApache2-blue)](LICENSE) [![GitLab Status](https://gitlab.parity.io/parity/substrate/badges/master/pipeline.svg)](https://gitlab.parity.io/parity/substrate/pipelines) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](docs/CONTRIBUTING.adoc)

<p align="center">
  <img src="/docs/media/sub.gif">
</p>


![Transx](https://avatars3.githubusercontent.com/u/58466741?s=400&u=b0649e38ddfc99730b975a5bdd0fa64f5324c49d&v=4)

## Introduction

   * Digital currency aggregation payment platform.
   * TransX leads the aggregate payment of digital currencies.
   * TransX provides great DCEP supporting services.

## Building

* Install Rust
    `curl https://sh.rustup.rs -sSf | sh`
    `rustup default stable`

* Install all the required dependencies with a single command.
    `curl https://getsubstrate.io -sSf | bash -s -- --fast`

* Wasm Compilation
    ```buildoutcfg
    rustup update nightly
    rustup target add wasm32-unknown-unknown --toolchain nightly
    ```
    >>> Transx uses WebAssembly (Wasm), and you will need to configure your Rust compiler to use nightly to support this build target.

* Rustup Update
    `rustup update`
    >>> Transx always uses the latest version of Rust stable and nightly for compilation. ensure your Rust compiler is always up to date

## NetWork
* Connect to sword(test network).

    `./target/release/transx --chain=sword`
    >>> Up to now, we only start the testnet.

* Run as dev.
    Remove the db
    `./target/release/transx purge-chain --dev`
    Start a development chain
    `./target/release/transx --dev`
* Run as local
    If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units.

    You'll need two terminal windows open.

    We'll start Alice's substrate node first on default TCP port 30333 with her chain database stored locally at /tmp/alice. The bootnode ID of her node is 12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp, which is generated from the --node-key value that we specify below:
    ```
    ./target/release/transx \
      --base-path /tmp/alice \
      --chain local \
      --alice \
      --node-key 0000000000000000000000000000000000000000000000000000000000000001
    ```

    In the second terminal, we'll start Bob's substrate node on a different TCP port of 30334, and with his chain database stored locally at /tmp/bob. We'll specify a value for the --bootnodes option that will connect his node to Alice's bootnode ID on TCP port 30333:
    ```
    ./target/release/transx \
      --base-path /tmp/bob \
      --chain local \
      --bob \
      --port 30334 \
      --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
    ```

    Additional CLI usage options are available and may be shown by running cargo run -- --help.
## Contributions & Code of Conduct

Please follow the contributions guidelines as outlined in [`docs/CONTRIBUTING.adoc`](docs/CONTRIBUTING.adoc). In all communications and contributions, this project follows the [Contributor Covenant Code of Conduct](docs/CODE_OF_CONDUCT.adoc).

## Security

The security policy and procedures can be found in [`docs/SECURITY.md`](docs/SECURITY.md).

## License

- Transx Primitives (`sp-*`), Frame (`frame-*`) and the pallets (`pallets-*`), binaries (`/bin`) and all other utilities are licensed under [Apache 2.0](LICENSE-APACHE2).
- Transx Client (`/client/*` / `sc-*`) is licensed under [GPL v3.0 with a classpath linking exception](LICENSE-GPL3).

