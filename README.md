## build

```bash
cargo build-sbf -- -Znext-lockfile-bump
```

## test

```bash
cargo test-sbf
```

## deploy program on local validator

check solana version

```bash
~ solana -V 

solana-cli 2.1.22 (src:26944979; feat:1416569292, client:Agave)
```

generate a new keypair and airdrop 20 SOL

```bash
solana-keygen new --outfile ~/.config/solana/id.json
solana airdrop 20
solana balance
```

deploy program

```bash
solana program deploy ./target/deploy/learn_solana_program.so
```

get program id from the deploy keypair

```bash
solana address -k ./target/deploy/learn_solana_program-keypair.json
```

res : CaDTBCo9DUTVT8AT3MB4taKBV6fXvQbPuWm4TgKFVMtZ

run client

```bash
cargo run --bin client
```
