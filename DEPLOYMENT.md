# pWrap deployment runbook

This runbook prepares and verifies a deployment. It does not authorize one.

## Fixed identity

```text
Program ID:       pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2
Upstream baseline 81adb66daa1405eb1568af8b74f5c30924655bd6
Program artifact  target/deploy/spl_token_wrap.so
Host Rust:        1.93.1
Solana CLI:       4.1.0
SBF builder:      cargo-build-sbf 4.1.0
SBF compiler:     platform-tools v1.54 / rustc 1.89.0
```

The 2026-08-17 preparation build used the official Agave `v4.1.0` aarch64 macOS archive with SHA-256 `331878e4a36689faf2c5bfe769481b9506c0158e43fc553f61c7b842cfbd4a86`. That archive reports `cargo-build-sbf 4.1.0`, `platform-tools v1.54`, and SBF `rustc 1.89.0`; the repository's Rust `1.93.1` is the host toolchain, not the ELF compiler. Its candidate pWrap ELF is 436,360 bytes with SHA-256 `fb64746885e19cd5a8a1f4f40c8a6dfff8183cc3d8e30b40a120a1d4dee7eb49`. Reproduce these values from the committed source before deployment; this record alone is not authorization.

One program keypair may reserve the same address on devnet, testnet, and mainnet because the ledgers are independent. The keypair is needed to create the program account on each cluster. It is not the payer and it is not the upgrade authority.

## Key boundaries

Three signing roles must remain explicit:

1. **Program-account keypair** — fixes the public program address. Keep it offline or sealed and never commit it.
2. **Fee payer** — funds rent and transaction fees. A funded operator wallet may fill this role without becoming program authority.
3. **Upgrade authority** — can replace the deployed code and therefore controls the PDA-held escrow and wrapped-mint authority indirectly. Do not default this role to the payer. Select and record it before deployment.

Before any use, restrict local key files to the owner:

```bash
chmod 600 "$PWRAP_PROGRAM_KEYPAIR" "$PWRAP_FEE_PAYER_KEYPAIR" "$PWRAP_UPGRADE_AUTHORITY_KEYPAIR"
```

## 1. Reproduce the source inputs

```bash
git status --porcelain=v1 --untracked-files=all
test -z "$(git status --porcelain=v1 --untracked-files=all)"
git rev-parse --verify HEAD
git rev-parse HEAD^{tree}
git show --no-patch --format='%H %cI %s' 81adb66daa1405eb1568af8b74f5c30924655bd6
rustc --version
rustup run nightly-2026-01-22 rustc --version
solana --version
cargo build-sbf --version
pnpm --version
```

The first `test` is a hard gate: staged, unstaged, and untracked source must all be absent. Required versions are host Rust `1.93.1`, Rust nightly `nightly-2026-01-22` for test/lint gates, Solana CLI and `cargo-build-sbf` `4.1.0`, `platform-tools v1.54` with its SBF `rustc 1.89.0`, and pnpm `10.15.1`. A different SBF builder or platform-tools compiler is a different artifact provenance and cannot close the deployment gate.

## 2. Regenerate and verify source contracts

```bash
pnpm install --frozen-lockfile
pnpm generate:clients
test -z "$(git status --porcelain=v1 --untracked-files=all)"
cargo check --workspace --all-targets --locked
pnpm --dir clients/js test
pnpm --dir clients/js build
```

The second clean-tree gate covers modified, staged, and newly generated files. It proves that regeneration matches committed `idl.json` and client source exactly. The generated IDL and clients must contain the pWrap address and all seven program instructions, including `SetCanonicalPointer`.

## 3. Verify the program key without exposing it

```bash
./scripts/verify-program-keypair.sh "$PWRAP_PROGRAM_KEYPAIR"
```

The script prints only the public address. It fails if the key does not derive the fixed pWrap address, has group/other permissions, or is tracked by this repository.

## 4. Build the deployable artifact

```bash
make build-sbf-program
make build-sbf-program-test-metadata-owner
make build-sbf-program-test-transfer-hook
make test-program
cargo test -p spl-token-wrap-cli --bin spl-token-wrap
cargo test -p spl-token-wrap-cli --test runner
test -s target/deploy/spl_token_wrap.so
test -z "$(git status --porcelain=v1 --untracked-files=all)"
shasum -a 256 target/deploy/spl_token_wrap.so
shasum -a 256 Cargo.lock
```

The final clean-tree gate catches any manifest, lockfile, generated-source, or other tracked input mutation made during the build. Record the exact ELF SHA-256, byte length, source commit and tree, `Cargo.lock` SHA-256, host Rust version, Solana CLI version, `cargo-build-sbf`/platform-tools/SBF-rustc versions, release-archive hash, and build host identity in the deployment evidence. Build independently a second time in the release environment and require byte-for-byte agreement.

## 5. Prove the target cluster

Use a reviewed QuickNode endpoint. Never use a public `api.*.solana.com` endpoint for this workflow.

```bash
solana genesis-hash --url "$PWRAP_RPC_URL"
solana program show --url "$PWRAP_RPC_URL" pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2
solana balance --url "$PWRAP_RPC_URL" --keypair "$PWRAP_FEE_PAYER_KEYPAIR"
solana address --keypair "$PWRAP_UPGRADE_AUTHORITY_KEYPAIR"
```

Pinned genesis hashes:

| Cluster | Genesis hash |
| --- | --- |
| devnet | `EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG` |
| testnet | `4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY` |
| mainnet | `5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d` |

Stop if the observed hash does not match the intended cluster. An existing program account at the pWrap address is also a stop condition until its owner, ProgramData, authority, slot, and ELF hash are reconciled.

Read-only QuickNode preflight on 2026-08-17 at finalized devnet slot `484882312` observed the expected devnet genesis hash, no account at the pWrap address, and executable Token-2022 and ZK ElGamal proof programs. Recheck all of those facts immediately before an authorized deployment.

## 6. Deployment command — explicit authorization required

Do not run this command during readiness work:

```bash
solana program deploy \
  --url "$PWRAP_RPC_URL" \
  --use-rpc \
  --program-id "$PWRAP_PROGRAM_KEYPAIR" \
  --fee-payer "$PWRAP_FEE_PAYER_KEYPAIR" \
  --upgrade-authority "$PWRAP_UPGRADE_AUTHORITY_KEYPAIR" \
  --commitment finalized \
  --output json \
  target/deploy/spl_token_wrap.so
```

First deployment target: devnet. Testnet and mainnet require separate approvals and repeat the entire cluster, authority, balance, artifact, and post-deployment verification gate.

After pWrap is finalized and KTEST exists, its mint authority may publish pWrap as the preferred deployment. This is a separate, explicit transaction and is not part of program deployment:

```bash
spl-token-wrap \
  --url "$PWRAP_RPC_URL" \
  --fee-payer "$PWRAP_FEE_PAYER_KEYPAIR" \
  set-canonical-pointer "$KTEST_MINT_ADDRESS" pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2 \
  --mint-authority "$KTEST_MINT_AUTHORITY_KEYPAIR"
```

The CLI validates the on-chain mint authority and existing pointer account, then atomically funds only the missing rent and writes the pointer. The KTEST sponsor payer must remain distinct from the mint authority.

## 7. Post-deployment proof

After an authorized deployment, independently verify:

- finalized program and ProgramData accounts;
- upgrade authority equals the approved address;
- live ELF hash equals the approved artifact hash;
- pWrap IDL and Rust/JavaScript clients derive identical PDAs;
- `CreateMint`, `Wrap`, `Unwrap`, and every unhappy-path invariant on a valueless devnet fixture;
- Token-2022 confidential configuration and a complete public-wrap → confidential-deposit → confidential-transfer → withdraw → unwrap lifecycle.

Do not describe the program as deployed, audited, production-ready, or safe for real value until the corresponding evidence exists.
