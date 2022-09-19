## testing

```sh
solana program dump \
  ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL \
  target/deploy/spl_associated_token_account.so \
  -um
solana program dump \
  TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA \
  target/deploy/spl_token.so \
  -um
RUST_BACKTRACE=1 cargo test-bpf
```