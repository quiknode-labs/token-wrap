# pWrap Mainnet deployment record

This is the immutable public evidence record for the initial pWrap Mainnet deployment. It records the loader deployment only. K256 did not initialize a wrapped-mint relationship, canonical pointer, escrow, or user token account.

## Live program

| Field | Finalized value |
| --- | --- |
| Cluster genesis | `5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d` |
| Program ID | `pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2` |
| ProgramData | `DJ7bADfr6LxWQsyzRGJXRwieXB4CmkRioTLpMoPvMhW1` |
| Upgrade authority | `EDJwdLWCgUdFMdcxqXNYjvB9EvF6qkAwcwYdwzjgU4TC` |
| Deployment slot | `440055335` |
| Deployment time | `2026-08-18T12:12:25Z` |
| Deployment transaction | [`5Sz8UEPDB6PxJzcmNNfGSGxV8dEmk5ixF7p1iLNbrKkB9H5orwsg4Jo5rzH8GTViVioH81TyRdvP2f7r691PuxP`](https://explorer.solana.com/tx/5Sz8UEPDB6PxJzcmNNfGSGxV8dEmk5ixF7p1iLNbrKkB9H5orwsg4Jo5rzH8GTViVioH81TyRdvP2f7r691PuxP) |
| Loader | `BPFLoaderUpgradeab1e11111111111111111111111` |
| Program account | Executable; 36 bytes; `1,141,440` lamports |
| ProgramData account | Non-executable loader data; 436,405 bytes; `3,038,269,680` lamports |
| Live ELF | 436,360 bytes; SHA-256 `fb64746885e19cd5a8a1f4f40c8a6dfff8183cc3d8e30b40a120a1d4dee7eb49` |
| Finalized verification | Live dump byte-identical to the approved ELF; authority present; named buffer consumed |
| Delayed visibility recheck | Finalized slot `440056798`; authority, deployment slot, and exact live bytes unchanged |

The ProgramData address is the same string as on Devnet because it is derived from the same program ID under the same loader. The accounts and authority state remain independent per cluster.

## Source and build provenance

| Input | Value |
| --- | --- |
| K256 fork commit | `528570840f8f9aff149677a62b7605986421e9c6` |
| Git tree | `cba4b360bbc4f555696b558768fe9173cce3ff2e` |
| Program-equivalent Devnet deployment commit | `e01a2be705d640aa71dadcffd9c5115dc0db71dc` |
| Upstream baseline | `81adb66daa1405eb1568af8b74f5c30924655bd6` |
| `Cargo.lock` SHA-256 | `3d508b3ad4e5ded45b79d24cdaf65b6befd8f1911024444ba71b2173df1e7bef` |
| Host Rust | `1.93.1` |
| Test/lint nightly | `nightly-2026-01-22` |
| Solana CLI / SBF builder | `4.1.0` |
| SBF compiler | `platform-tools v1.54`, SBF `rustc 1.89.0` |
| Agave release archive SHA-256 | `331878e4a36689faf2c5bfe769481b9506c0158e43fc553f61c7b842cfbd4a86` |

The deployable program inputs are unchanged between the program-equivalent Devnet commit and the Mainnet source commit. A clean detached build reproduced the retained Devnet ELF byte-for-byte. The program test suite executed 115 tests successfully; one non-default compliance-customizer test remained intentionally ignored. The staged loader-buffer dump and the finalized live-program dump both matched the same approved ELF.

## Ceremony evidence and cost

| Field | Value |
| --- | --- |
| Fee payer | `3WzAUVNX5tm9gJWS2Y2Vg3wMnAtTm23fFX2jHaYy67bV` |
| Named loader buffer | `6HPiyVk5Ej91uw1jzHVHx7cywvzUThUzU7fzqQwRf3sk` |
| Buffer authority | `EDJwdLWCgUdFMdcxqXNYjvB9EvF6qkAwcwYdwzjgU4TC` |
| First buffer transaction | [`5ifcnnG6oq9eHdFazWzTo6BRT9WPwEcJTPhrv5zjDWDVyZPwamMKcZekEUPbFrfeuz9ep3jSgpm1nB4MKo3XDeZQ`](https://explorer.solana.com/tx/5ifcnnG6oq9eHdFazWzTo6BRT9WPwEcJTPhrv5zjDWDVyZPwamMKcZekEUPbFrfeuz9ep3jSgpm1nB4MKo3XDeZQ) |
| Ceremony slots | `440055082` through `440055335` |
| Transactions | 479 finalized, 0 failed |
| Payer balance before | `12,152,205,783` lamports |
| Payer balance after | `9,107,999,663` lamports |
| Transaction fees | `4,795,000` lamports (`0.004795000 SOL`) |
| Persistent loader rent | `3,039,411,120` lamports (`3.039411120 SOL`) |
| Exact payer delta | `3,044,206,120` lamports (`3.044206120 SOL`) |

The buffer held `3,038,214,000` lamports while the ELF was staged. The final loader transaction consumed the buffer and moved the required balance into the Program and ProgramData accounts. A finalized read found the buffer account absent afterward.

## Runtime preflight

Immediately before the ceremony, finalized reads proved:

- the Mainnet genesis hash matched the value above;
- the Program, deterministic ProgramData address, and named buffer were absent;
- legacy Token, Token-2022, and the ZK ElGamal proof program were executable under their expected loaders;
- the Confidential Transfer proof enable/temporary-disable/re-enable sequence left proof execution enabled; and
- the feature that disables deployment of SBPF v0/v1/v2 programs was inactive. The approved ELF is SBPFv0.

The CLI feature verifier remained enabled. No skip-feature-verification, skip-preflight, finalization, warning-bypass, or close option was used.

## State and initialization

pWrap has no global initialization instruction or configuration account. Loader deployment was the complete program-level initialization.

Each token relationship remains separate and permissionless: `CreateMint` establishes a wrapped mint and backpointer; escrow creation is separate; `Wrap` and `Unwrap` operate on that relationship; and `SetCanonicalPointer` is an optional transaction authorized by the underlying mint authority. K256 submitted none of those instructions on Mainnet during this deployment.

At both the first finalized read and the delayed slot `440056798` recheck, the only signature indexed for the pWrap address was the loader deployment transaction above. This is point-in-time evidence, not an allowlist: the deployed program has no global pause or asset-admission gate, so third parties can invoke `CreateMint` for compatible existing mints without K256 approval.

## Security and operating truth

- This pWrap binary is distinct from the latest audited upstream snapshot and has not received an independent audit bound to this exact source and ELF.
- Historical upstream audits are lineage only. Their residual findings around freeze-authority coordination, absent default auditor, transparent wrap amounts, and partial metadata synchronization remain relevant.
- The raw upgrade authority can technically upgrade, rotate, finalize, or close ProgramData. K256 policy permanently forbids finalization and closure, but the loader does not cryptographically enforce that narrower rule.
- Mainnet deployment is not K256 admission, endorsement, or support for USDC, USDT, SOL/WSOL, or any other asset. No K256 Mainnet asset relationship was created.
- Independent sealed, restore-tested recovery for the Mainnet upgrade authority remains an operator follow-up. Losing that authority would make future upgrades impossible; the program-account key cannot recover it.

Every future Mainnet change must follow [`MAINNET_RUNBOOK.md`](./MAINNET_RUNBOOK.md), preserve the literal program address, and produce a new point-in-time evidence record.
