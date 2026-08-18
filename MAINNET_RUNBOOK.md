# pWrap Mainnet upgrade runbook

This runbook is for future upgrades of the already deployed Mainnet program. It does not authorize a signing event by itself. Record an explicit approval for the exact source, artifact, authority, payer, and cluster before every upgrade.

The initial Mainnet deployment is complete and must never be rerun. Its immutable evidence is [`MAINNET_DEPLOYMENT.md`](./MAINNET_DEPLOYMENT.md).

## Fixed Mainnet identity

```text
Program ID:        pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2
ProgramData:       DJ7bADfr6LxWQsyzRGJXRwieXB4CmkRioTLpMoPvMhW1
Mainnet genesis:   5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d
Upgrade authority: EDJwdLWCgUdFMdcxqXNYjvB9EvF6qkAwcwYdwzjgU4TC
```

Every upgrade targets the literal public program ID. Never pass the program-account keypair for an upgrade, never deploy a replacement identity, and never treat another cluster's authority or evidence as Mainnet state.

## Permanent lifecycle rules

- Never run `solana program close` against the pWrap Program or ProgramData account.
- Never use `--final`, remove the upgrade authority, rotate it to `None`, or otherwise make pWrap immutable.
- Never use a warning-bypass, feature-verification bypass, or preflight bypass.
- Use a fresh named buffer keypair for one ceremony. Keep it until all ambiguous results are reconciled; archive it after the finalized ProgramData state and live bytes are proven.
- Keep the fee payer, upgrade authority, and temporary buffer roles explicit and separate. A payer never becomes authority implicitly.
- Use a fresh operational fee payer whose secret has never been committed to source control. The historical initial-deployment payer is evidence, not a future default.
- Prove an independently sealed recovery copy of the upgrade authority before relying on it. Authority loss makes future upgrades impossible.

The upgradeable loader does not distinguish “may upgrade” from “may close.” With a raw authority key, never-close is an operator and custody invariant rather than an on-chain guarantee. Moving to governance requires a separately reviewed controller and authority-rotation ceremony.

## Required environment

Use role-based environment variables; never publish key paths, key bytes, recovery locations, or RPC credentials.

```bash
test -n "$PWRAP_RPC_URL"
test -n "$PWRAP_FEE_PAYER_KEYPAIR"
test -n "$PWRAP_UPGRADE_AUTHORITY_KEYPAIR"
test -n "$PWRAP_BUFFER_KEYPAIR"
test -n "$PWRAP_SOLANA"
test -n "$PWRAP_CARGO_BUILD_SBF"

case "$PWRAP_SOLANA" in /*) ;; *) exit 1 ;; esac
case "$PWRAP_CARGO_BUILD_SBF" in /*) ;; *) exit 1 ;; esac
test -x "$PWRAP_SOLANA"
test -x "$PWRAP_CARGO_BUILD_SBF"

PWRAP_PROGRAM_ID=pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2
PWRAP_PROGRAMDATA=DJ7bADfr6LxWQsyzRGJXRwieXB4CmkRioTLpMoPvMhW1
PWRAP_MAINNET_GENESIS=5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d
PWRAP_MAINNET_AUTHORITY=EDJwdLWCgUdFMdcxqXNYjvB9EvF6qkAwcwYdwzjgU4TC
```

Set the two tool variables to explicit absolute paths in the private Mainnet operator profile. Never derive them from the current shell's `PATH`. The approved CLI and `cargo-build-sbf` must both be version `4.1.0`; the SBF toolchain must be `platform-tools v1.54` with SBF `rustc 1.89.0`.

## 1. Approve and reproduce the release

1. Record the exact source commit and tree, `Cargo.lock` hash, full toolchain, artifact length and SHA-256, and the security-review scope.
2. Build twice from clean detached source with `--locked`; require byte-identical ELFs.
3. Regenerate the IDL and clients and require a clean Git tree.
4. Run the full program, CLI unit/integration, and JavaScript test gates.
5. Review ABI and persistent-PDA compatibility, migrations, rollback or forward-fix behavior, and every accepted audit finding.
6. Retain the currently deployed source and exact live ELF as the recovery candidate.

No K256 asset mapping or customer support claim is implied by an executable deployment. The program remains permissionless and has no asset allowlist.

## 2. Prove Mainnet and current authority

Use a reviewed private Mainnet RPC endpoint. Public `api.*.solana.com` endpoints are forbidden for this workflow.

```bash
test "$("$PWRAP_SOLANA" --version | awk '{print $2}')" = 4.1.0
test "$("$PWRAP_CARGO_BUILD_SBF" --version | sed -n '1p')" = \
  'cargo-build-sbf 4.1.0'
test "$("$PWRAP_CARGO_BUILD_SBF" --version | sed -n '2p')" = \
  'platform-tools v1.54'
test "$("$PWRAP_CARGO_BUILD_SBF" --version | sed -n '3p')" = \
  'rustc 1.89.0'
test "$("$PWRAP_SOLANA" genesis-hash --url "$PWRAP_RPC_URL")" = \
  "$PWRAP_MAINNET_GENESIS"
test "$("$PWRAP_SOLANA" address \
  --keypair "$PWRAP_UPGRADE_AUTHORITY_KEYPAIR")" = \
  "$PWRAP_MAINNET_AUTHORITY"

"$PWRAP_SOLANA" program show \
  --url "$PWRAP_RPC_URL" \
  --commitment finalized \
  --output json \
  "$PWRAP_PROGRAM_ID"
```

Stop unless the live program is executable under the upgradeable loader, its ProgramData address is exact, and its authority is the approved Mainnet authority. Dump the current live program, record its full hash and length, and bind it to the retained recovery source before writing a new buffer.

Recheck Token, Token-2022, ZK ElGamal, Confidential Transfer feature state, and every feature that can reject the new ELF. Do not skip the CLI's feature verification.

## 3. Stage and prove a fresh named buffer

```bash
PWRAP_NEW_ELF=target/deploy/spl_token_wrap.so
PWRAP_NEW_ELF_SIZE="$(stat -f '%z' "$PWRAP_NEW_ELF")"

"$PWRAP_SOLANA" program write-buffer \
  --url "$PWRAP_RPC_URL" \
  --use-rpc \
  --buffer "$PWRAP_BUFFER_KEYPAIR" \
  --buffer-authority "$PWRAP_UPGRADE_AUTHORITY_KEYPAIR" \
  --fee-payer "$PWRAP_FEE_PAYER_KEYPAIR" \
  --max-len "$PWRAP_NEW_ELF_SIZE" \
  --commitment finalized \
  --output json \
  "$PWRAP_NEW_ELF"
```

At finalized commitment, show the named buffer and require its authority to equal the approved Mainnet authority. Dump it independently, require byte-for-byte equality with the approved ELF, and record both hashes and lengths.

If any chunk write times out, query the named buffer before retrying. Never overwrite or close a buffer while the result is ambiguous.

## 4. Upgrade the existing address

Only after the buffer proof is complete:

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

Do not add a program filepath when deploying from the already verified named buffer. Do not add `--final`.

An error or timeout is not proof of failure. Before any retry, query the literal pWrap address, ProgramData, authority, deployment slot, live bytes, and buffer at finalized commitment. If the new bytes already landed, do not resubmit.

## 5. Finalized acceptance

After the upgrade:

1. Require the transaction to be finalized with `err: null`.
2. Require the same literal Program ID and ProgramData address.
3. Require the authority to remain `EDJwdLWCgUdFMdcxqXNYjvB9EvF6qkAwcwYdwzjgU4TC` unless a separately approved rotation was part of the ceremony.
4. Dump the live program. Require the approved ELF as an exact prefix and require every trailing ProgramData byte to be zero. Equal-length dumps must hash exactly to the artifact.
5. Record the source/tree/lock/toolchain/artifact identities, transaction, slot, authority, account sizes, live dump, buffer result, fee payer, cost, and all verification results.
6. Repeat finalized reads after a visibility delay and run the full approved behavioral and unhappy-path suite on a local validator plus non-submitting Mainnet simulations. Do not create or mutate a Mainnet asset relationship without a separate asset-specific authorization.
7. Create a new immutable deployment evidence record and update the README cluster matrix.

A larger ELF may extend ProgramData. A smaller ELF does not shrink it; the loader zero-fills unused capacity. Check capacity before upgrading and never equate a padded full-dump hash with the smaller artifact hash.

## Authority rotation

Rotation is a separate high-risk operation. Prove the new authority's independently recoverable signer before handoff, require both current and new authorities, and verify the finalized ProgramData authority afterward. Never rotate to `None` and never use a finalization option. Update the public authority record in the same change.
