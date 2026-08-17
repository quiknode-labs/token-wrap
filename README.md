# k256 pWrap

[![CI](https://github.com/quiknode-labs/token-wrap/actions/workflows/main.yml/badge.svg)](https://github.com/quiknode-labs/token-wrap/actions/workflows/main.yml)

pWrap is the k256-maintained fork of the Solana Program Token Wrap implementation. It converts an existing SPL Token or Token-2022 mint into a deterministic wrapped representation backed by on-chain escrow.

## Deployment identity

| Item                     | Value                                                                                                                                      |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------ |
| pWrap program ID         | `pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2`                                                                                              |
| Deployment scope         | Devnet only                                                                                                                                |
| Devnet genesis           | `EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG`                                                                                             |
| Devnet upgrade authority | `G1kdS4CCCKZFKzupAm9N5ZLMvx5bgfzpUx9xkmt1KxYR`                                                                                             |
| K256 fork                | <https://github.com/quiknode-labs/token-wrap>                                                                                              |
| Upstream                 | <https://github.com/solana-program/token-wrap>                                                                                             |
| Upstream baseline        | [`81adb66daa1405eb1568af8b74f5c30924655bd6`](https://github.com/solana-program/token-wrap/commit/81adb66daa1405eb1568af8b74f5c30924655bd6) |
| IDL                      | [`idl.json`](./idl.json)                                                                                                                   |

`pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2` is the permanent devnet address. The initial deployment uses the matching program-account keypair; every later upgrade targets the literal public address and does not replace it. Testnet and mainnet deployment are outside this fork's current authorization and runbook.

The upstream `TwRapQCDhWkZRrDaHfZGuHxkZ91gHDRkyuzNqeU5MgR` address is not used by pWrap. Changing the program ID changes every program-derived wrapped mint, authority, backpointer, canonical pointer, and escrow address.

The program-account and upgrade-authority keypairs are operator key material. They must stay outside this public repository. The program-account keypair is retained for identity recovery, but it is not the fee payer or upgrade authority and is not needed for normal devnet upgrades.

## Permanent devnet identity

All Rust, IDL, JavaScript, CLI, and PDA derivations remain pinned to the pWrap address above. The supported lifecycle is initial deploy once, then upgrade that same address. A replacement program ID is not a pWrap upgrade.

The program must remain upgradeable, so neither deployment with `--final` nor removal of the upgrade authority is allowed. Any `solana program close` operation targeting the pWrap Program or ProgramData accounts is forbidden. `CloseStuckEscrow` is unrelated: it can close only a zero-balance token escrow ATA after validating the documented stuck-escrow conditions; it cannot close the program.

The upgradeable loader does not provide a permission that allows upgrades while cryptographically forbidding closure. A raw upgrade-authority signer can technically upgrade, rotate, finalize, or close the program. Until that authority is transferred to a governance controller that enforces a narrower policy, “never close” is a key-custody and operator invariant. Authority loss makes future upgrades impossible; compromise exposes the program-controlled escrow and issuance logic.

## Provenance and security status

This fork deliberately starts from the latest upstream `main`, not the older released program snapshot:

- latest upstream program release: `program@v1.0.0` at `4e4e1d0` (2025-09-11);
- latest audit listed by upstream: Runtime Verification at `228dc97` (2025-10-30);
- fork baseline: `81adb66` (2026-08-17).

Upstream added `SetCanonicalPointer` and upgraded core Solana dependencies after the latest listed audit. The pWrap program-ID change also produces a distinct binary. The historical audits are useful lineage, not an audit of the current pWrap binary. Devnet experimentation does not authorize mainnet or real-value assets.

The latest Runtime Verification report leaves acknowledged findings around freeze-authority coordination, loss of an underlying confidential auditor policy, transparent wrap amounts, and partial metadata synchronization. pWrap also remains permissionless and upgradeable. The valueless KTEST experiment excludes freeze authority and broader asset admission, but does not claim those general risks are fixed.

## Protocol model

pWrap keeps one deterministic relationship between an unwrapped mint and a wrapped mint for a selected token program:

1. `CreateMint` creates the wrapped mint and its backpointer PDA.
2. `Wrap` transfers unwrapped tokens into the escrow ATA and mints the backed wrapped amount.
3. `Unwrap` burns wrapped tokens and atomically releases the same underlying amount from escrow.
4. `CloseStuckEscrow` closes a zero-balance escrow whose account extensions no longer match a recreated mint.
5. `SyncMetadataToToken2022` copies supported metadata into a wrapped Token-2022 mint.
6. `SyncMetadataToSplToken` copies supported metadata into the wrapped SPL Token mint's Metaplex account.
7. `SetCanonicalPointer` lets an unwrapped mint authority publish its preferred Token Wrap deployment.

The wrapped mint, mint authority, backpointer, canonical pointer, and escrow addresses are program-derived. Users retain ownership of their token accounts. The wrapper controls escrow release and wrapped issuance through its deployed code, so the program upgrade authority is a real economic authority and must be governed accordingly.

## Confidential Transfer policy

When the wrapped mint uses Token-2022, the default mint customizer initializes:

- `ConfidentialTransferMint` with automatic account approval;
- no confidential-transfer authority;
- no auditor;
- `MetadataPointer` controlled by the wrapped-mint authority PDA;
- decimals and freeze authority copied from the unwrapped mint.

pWrap itself implements public wrap and unwrap. Configure-account, deposit, confidential transfer, apply-pending-balance, and withdraw are Token-2022 operations and use the relevant proof programs. Moving into or out of pWrap escrow is public; the wrapper does not make the boundary transaction confidential.

K256 will create, sponsor, canonicalize, document, and test only an exact fee-free, non-rebasing, non-hooked, non-freezable valueless devnet fixture. The permissionless program has no asset allowlist, so this operating policy cannot prevent third-party invocation. Broader K256 asset support is a separate security and product decision.

## Build and test

Required toolchains are pinned in the repository:

- Rust `1.93.1` (`rust-toolchain.toml`);
- Rust nightly `nightly-2026-01-22` for the repository test/lint gates (`Makefile`);
- Solana CLI `4.1.0` (`Cargo.toml` workspace metadata and `Makefile`);
- pnpm `10.15.1` (`package.json`).

```bash
pnpm install --frozen-lockfile
pnpm generate:clients
cargo check --workspace --all-targets --locked
make build-sbf-program
make build-sbf-program-test-metadata-owner
make build-sbf-program-test-transfer-hook
make test-program
cargo test -p spl-token-wrap-cli --bin spl-token-wrap
cargo test -p spl-token-wrap-cli --test runner
pnpm --dir clients/js test
pnpm --dir clients/js build
```

`pnpm generate:clients` regenerates `idl.json` and the JavaScript client from `program/idl.ts`. The generated client packages retain their upstream-compatible names but are marked private in this fork; no K256 package publication contract exists yet.

Unlike the upstream snapshot, the generated contract and CLI both expose `SetCanonicalPointer`. The JavaScript test is no longer a no-op: it verifies the pWrap identity, all seven discriminators, the instruction/account codecs, and a canonical-pointer PDA against an independent Solana CLI derivation.

For initial deployment, future upgrades, key boundaries, artifact checks, cluster proof, and permanent-address safeguards, see [`DEPLOYMENT.md`](./DEPLOYMENT.md).

## Upstream maintenance

The local checkout uses `origin` for the k256 fork and `upstream` for the Solana Program repository. Review upstream changes before merging them; program behavior, IDL, generated clients, audits, and the pWrap address must remain one coherent release.

```bash
git fetch upstream --tags
git log --oneline --left-right main...upstream/main
git merge upstream/main
```

After every upstream merge, restore and verify the pWrap program ID, regenerate clients, run the full checks above, and produce a new binary hash and security review scope. An upstream merge is deployed only as an upgrade of the existing devnet address.

## Upstream audit lineage

| Auditor              | Date       | Upstream version                                                                                        | Report                                                                                                                                                |
| -------------------- | ---------- | ------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Zellic               | 2025-05-16 | [`75c5529`](https://github.com/solana-program/token-wrap/tree/75c5529d5a191f12bd58b6b92ca0104ce3464763) | [PDF](https://github.com/anza-xyz/security-audits/blob/2294fc0e61c153c8aed174e9f63a1730683f1f2a/spl/ZellicTokenWrapAudit-2025-05-16.pdf)              |
| Runtime Verification | 2025-06-11 | [`dd71fc1`](https://github.com/solana-program/token-wrap/tree/dd71fc10c651b07b7d62b151021216e5321b1789) | [PDF](https://github.com/anza-xyz/security-audits/blob/2294fc0e61c153c8aed174e9f63a1730683f1f2a/spl/RuntimeVerificationTokenWrapAudit-2025-06-11.pdf) |
| Runtime Verification | 2025-10-30 | [`228dc97`](https://github.com/solana-program/token-wrap/tree/228dc976d454b766e649ea7759304e1fb457c76d) | [PDF](https://github.com/anza-xyz/security-audits/blob/80287adb867b83a394d62dd7ab88a693eb266539/spl/RuntimeVerificationTokenWrapAudit-2025-10-30.pdf) |

## License

Apache License 2.0. See [`LICENSE`](./LICENSE). Upstream history and attribution are preserved by the GitHub fork.
