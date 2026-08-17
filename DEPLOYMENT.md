# pWrap deployment runbook

This is the only supported runbook for the initial devnet deployment and every future devnet upgrade. It defines the operation but does not itself authorize a signing event; record a separate explicit approval for each deployment or upgrade. Testnet and mainnet are outside scope and are not authorized by this document.

## Fixed identity

```text
Program ID:       pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2
Devnet genesis:   EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG
Upgrade authority G1kdS4CCCKZFKzupAm9N5ZLMvx5bgfzpUx9xkmt1KxYR
Upstream baseline 81adb66daa1405eb1568af8b74f5c30924655bd6
Program artifact  target/deploy/spl_token_wrap.so
Host Rust:        1.93.1
Solana CLI:       4.1.0
SBF builder:      cargo-build-sbf 4.1.0
SBF compiler:     platform-tools v1.54 / rustc 1.89.0
```

The 2026-08-17 preparation build used the official Agave `v4.1.0` aarch64 macOS archive with SHA-256 `331878e4a36689faf2c5bfe769481b9506c0158e43fc553f61c7b842cfbd4a86`. That archive reports `cargo-build-sbf 4.1.0`, `platform-tools v1.54`, and SBF `rustc 1.89.0`; the repository's Rust `1.93.1` is the host toolchain, not the ELF compiler. Its candidate pWrap ELF is 436,360 bytes with SHA-256 `fb64746885e19cd5a8a1f4f40c8a6dfff8183cc3d8e30b40a120a1d4dee7eb49`. Reproduce these values from the committed source before deployment; this record alone is not authorization.

The pWrap address is permanent on devnet. The matching program keypair is needed only for the first deployment. All later upgrades target the literal program ID, preserve its ProgramData relationship, and use the approved upgrade authority. The program keypair is not the payer and is not the upgrade authority.

## Permanent-address and never-close contract

- Deploy once to `pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2`; upgrade that address thereafter. Never deploy a replacement identity.
- Never close the pWrap Program account or its ProgramData account. Never use the loader's warning-bypass option against either account.
- Never deploy with `--final`, remove the upgrade authority, or otherwise make the program immutable.
- Keep the Rust declaration, IDL, generated clients, CLI, PDA tests, and deployment evidence pinned to the same address.
- `CloseStuckEscrow` closes only an empty token escrow ATA after program validation. It has no loader authority and cannot close pWrap itself.

The upgradeable loader cannot enforce “may upgrade but may never close” for a raw authority key: the authority can technically upgrade, rotate, finalize, or close. Until governance takes the authority, this contract is enforced through custody and operator policy. An on-chain guarantee would require transferring authority to a controller whose policy cannot authorize closure.

## Key boundaries

Three signing roles must remain explicit:

1. **Program-account keypair** — fixes the public program address. Keep it offline or sealed and never commit it.
2. **Fee payer** — funds rent and transaction fees. A funded operator wallet may fill this role without becoming program authority.
3. **Upgrade authority** — the dedicated key whose public address is `G1kdS4CCCKZFKzupAm9N5ZLMvx5bgfzpUx9xkmt1KxYR`. It can replace deployed code and therefore controls the PDA-held escrow and wrapped-mint authority indirectly. It must never default to the payer.

Keep independently sealed recovery copies of both the program-account and upgrade-authority keypairs. Before first deployment and after every recovery or authority rotation, derive each public address from the recovered key and compare it to this document. Losing the authority makes future upgrades impossible; the program keypair cannot recover or replace it. The payer is disposable and must not be used as either durable authority.

Use an explicit, Git-ignored buffer keypair for each deployment ceremony. The buffer is temporary loader state, not a fourth authority. Keeping its key makes partial writes resumable without relying on an ephemeral recovery phrase. Never overwrite or clean up a buffer while a deployment result is ambiguous.

Before any use, restrict local key files to the owner:

```bash
chmod 600 \
  "$PWRAP_PROGRAM_KEYPAIR" \
  "$PWRAP_FEE_PAYER_KEYPAIR" \
  "$PWRAP_UPGRADE_AUTHORITY_KEYPAIR" \
  "$PWRAP_BUFFER_KEYPAIR"
```

## 1. Reproduce the source inputs

```bash
git status --porcelain=v1 --untracked-files=all
test -z "$(git status --porcelain=v1 --untracked-files=all)"
git rev-parse --verify HEAD
git rev-parse HEAD^{tree}
git show --no-patch --format='%H %cI %s' 81adb66daa1405eb1568af8b74f5c30924655bd6
PWRAP_SOLANA="$(command -v solana)"
rustc --version
rustup run nightly-2026-01-22 rustc --version
"$PWRAP_SOLANA" --version
cargo build-sbf --version
pnpm --version
test "$("$PWRAP_SOLANA" --version | awk '{print $2}')" = 4.1.0
```

The first `test` is a hard gate: staged, unstaged, and untracked source must all be absent. Required versions are host Rust `1.93.1`, Rust nightly `nightly-2026-01-22` for test/lint gates, Solana CLI and `cargo-build-sbf` `4.1.0`, `platform-tools v1.54` with its SBF `rustc 1.89.0`, and pnpm `10.15.1`. Confirm that `command -v solana` and `command -v cargo-build-sbf` resolve to the approved release. A different SBF builder or platform-tools compiler is a different artifact provenance and cannot close the deployment gate.

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

## 3. Verify durable keys without exposing them

```bash
./scripts/verify-program-keypair.sh "$PWRAP_PROGRAM_KEYPAIR"
test "$("$PWRAP_SOLANA" address --keypair "$PWRAP_UPGRADE_AUTHORITY_KEYPAIR")" = \
  G1kdS4CCCKZFKzupAm9N5ZLMvx5bgfzpUx9xkmt1KxYR
```

The script prints only the public program address. It fails if the key does not derive the fixed pWrap address, has group/other permissions, or is tracked by this repository. The second check proves that the available authority key matches the authority approved for devnet. Repeat both checks against each independently recovered copy before relying on it.

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
"$PWRAP_SOLANA" genesis-hash --url "$PWRAP_RPC_URL"
"$PWRAP_SOLANA" program show --url "$PWRAP_RPC_URL" pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2
"$PWRAP_SOLANA" balance --url "$PWRAP_RPC_URL" --keypair "$PWRAP_FEE_PAYER_KEYPAIR"
"$PWRAP_SOLANA" address --keypair "$PWRAP_UPGRADE_AUTHORITY_KEYPAIR"
```

The only allowed genesis hash is devnet: `EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG`. Stop if the observed value differs. An existing program account at the pWrap address is also a stop condition for the initial path until its owner, ProgramData, authority, slot, and ELF hash are reconciled. Testnet and mainnet endpoints must not be used with this runbook.

Read-only QuickNode preflight on 2026-08-17 at finalized devnet slot `484882312` observed the expected devnet genesis hash, no account at the pWrap address, and executable Token-2022 and ZK ElGamal proof programs. Recheck all of those facts immediately before an authorized deployment.

## 6. Initial devnet deployment

Run only after recording explicit approval. This is the only operation that uses the program-account keypair. Do not add `--final`.

First write and verify a named buffer. This isolates resumable chunk writes from the final Program/ProgramData creation transaction:

```bash
"$PWRAP_SOLANA" program write-buffer \
  --url "$PWRAP_RPC_URL" \
  --use-rpc \
  --buffer "$PWRAP_BUFFER_KEYPAIR" \
  --buffer-authority "$PWRAP_UPGRADE_AUTHORITY_KEYPAIR" \
  --fee-payer "$PWRAP_FEE_PAYER_KEYPAIR" \
  --commitment finalized \
  --output json \
  target/deploy/spl_token_wrap.so

PWRAP_BUFFER_ADDRESS="$("$PWRAP_SOLANA" address --keypair "$PWRAP_BUFFER_KEYPAIR")"
PWRAP_STAGED_ELF="$(mktemp)"
"$PWRAP_SOLANA" program show \
  --url "$PWRAP_RPC_URL" \
  --commitment finalized \
  "$PWRAP_BUFFER_ADDRESS"
"$PWRAP_SOLANA" program dump \
  --url "$PWRAP_RPC_URL" \
  --commitment finalized \
  "$PWRAP_BUFFER_ADDRESS" \
  "$PWRAP_STAGED_ELF"
cmp target/deploy/spl_token_wrap.so "$PWRAP_STAGED_ELF"
shasum -a 256 "$PWRAP_STAGED_ELF"
rm -f "$PWRAP_STAGED_ELF"
```

Only after the buffer is finalized and byte-identical to the approved ELF, create pWrap from that buffer:

```bash
"$PWRAP_SOLANA" program deploy \
  --url "$PWRAP_RPC_URL" \
  --use-rpc \
  --buffer "$PWRAP_BUFFER_KEYPAIR" \
  --program-id "$PWRAP_PROGRAM_KEYPAIR" \
  --fee-payer "$PWRAP_FEE_PAYER_KEYPAIR" \
  --upgrade-authority "$PWRAP_UPGRADE_AUTHORITY_KEYPAIR" \
  --commitment finalized \
  --output json
```

The deployment creates the upgradeable-loader Program and ProgramData accounts. pWrap has no global configuration account and needs no separate initialization transaction. Wrapped mints, backpointers, authorities, canonical pointers, and escrows are created later and only when their corresponding instructions are invoked.

Deployment is multi-transaction. After any timeout or ambiguous error, query pWrap at finalized commitment before retrying. If it exists, verify ProgramData, authority, slot, and live bytes and do not rerun the deploy. If pWrap is absent and the named buffer is still a valid loader buffer controlled by the approved authority, resume from that buffer. Never bulk-close authority buffers, and never close a specific buffer until the deployment state has been fully reconciled.

After the initial devnet deployment reaches finalized commitment and KTEST exists, its mint authority may optionally publish pWrap as the preferred deployment. This is a separate transaction and is not part of program deployment:

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
- live executable content matches the approved ELF prefix and any remaining ProgramData capacity contains only zero padding;
- pWrap IDL and Rust/JavaScript clients derive identical PDAs;
- `CreateMint`, `Wrap`, `Unwrap`, and every unhappy-path invariant on a valueless devnet fixture;
- Token-2022 confidential configuration and a complete public-wrap → confidential-deposit → confidential-transfer → withdraw → unwrap lifecycle.

Do not describe the program as deployed, audited, production-ready, or safe for real value until the corresponding evidence exists.

## 8. Future devnet upgrades

An upgrade preserves the pWrap Program account and all derived addresses. It replaces only the executable bytes in the existing ProgramData account. The program-account keypair is not used.

Before separately authorizing an upgrade:

1. Record the source commit and tree, `Cargo.lock` hash, complete toolchain, artifact length, and two matching clean-build hashes.
2. Capture finalized `solana program show` output and dump and hash the current live ELF. Retain its exact source and artifact as the recovery candidate.
3. Review account/ABI compatibility, persistent PDA layouts, instruction compatibility, migration needs, and rollback or forward-fix behavior.
4. Prove the devnet genesis hash and confirm the current ProgramData address and authority still match the recorded deployment.

Then write a named buffer and perform the same `program show`, `program dump`, byte comparison, and hash checks from section 6. Upgrade the existing literal address from that verified buffer; never pass the program-account keypair and never add `--final`:

```bash
"$PWRAP_SOLANA" program deploy \
  --url "$PWRAP_RPC_URL" \
  --use-rpc \
  --buffer "$PWRAP_BUFFER_KEYPAIR" \
  --program-id pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2 \
  --fee-payer "$PWRAP_FEE_PAYER_KEYPAIR" \
  --upgrade-authority "$PWRAP_UPGRADE_AUTHORITY_KEYPAIR" \
  --commitment finalized \
  --output json
```

Solana CLI 4.1.0 automatically extends ProgramData when a larger approved artifact requires it; do not disable that behavior. A smaller later ELF does not shrink ProgramData: the loader copies the ELF and zero-fills the remaining capacity. Therefore, record both lengths and the full dump hash, require the dump prefix to be byte-identical to the approved ELF, and require every trailing byte to be zero. Direct dump/artifact hash equality is valid only when their lengths are equal.

After the transaction reaches finalized commitment, repeat every proof in section 7, rerun the valueless KTEST lifecycle and unhappy paths, and record the transaction signature, deployment slot, authority, ProgramData address, source identity, artifact hash, full live dump hash, lengths, and result.

Authority rotation is a separate high-risk operation. Before any rotation, prove that the new signer is recoverable and record its public address. Require both the existing and new authority signers, verify the finalized ProgramData authority afterward, and update this document in the same change. Never rotate to `None` or use a finalization option.
