# pWrap devnet deployment record

This is the public evidence record for the permanent pWrap Devnet program deployment. Asset
selection and lifecycle testing are governed separately by
[`DEVNET_EXISTING_ASSET_TEST.md`](./DEVNET_EXISTING_ASSET_TEST.md), which accepts only an exact mint
that already exists independently of K256.

## Live program

| Item                              | Finalized value                                                                                                                                                                                                                      |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Cluster                           | Solana devnet                                                                                                                                                                                                                        |
| Genesis hash                      | `EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG`                                                                                                                                                                                       |
| Program                           | [`pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2`](https://explorer.solana.com/address/pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2?cluster=devnet)                                                                                      |
| Loader                            | `BPFLoaderUpgradeab1e11111111111111111111111`                                                                                                                                                                                        |
| ProgramData                       | `DJ7bADfr6LxWQsyzRGJXRwieXB4CmkRioTLpMoPvMhW1`                                                                                                                                                                                       |
| Upgrade authority                 | `G1kdS4CCCKZFKzupAm9N5ZLMvx5bgfzpUx9xkmt1KxYR`                                                                                                                                                                                       |
| Initial deploy slot               | `484920383`                                                                                                                                                                                                                          |
| Initial deploy time               | `2026-08-17T23:28:51Z`                                                                                                                                                                                                               |
| Post-deployment verification slot | `484925035`                                                                                                                                                                                                                          |
| Initial deploy transaction        | [`3czLqqm16e1wjdg89RoaKTKQk2cniKGXrL5rtefEhEYjL1dz2CSUQnUyipuCiucSFoSU5LCaoHFvMj6ZVnDSaMbB`](https://explorer.solana.com/tx/3czLqqm16e1wjdg89RoaKTKQk2cniKGXrL5rtefEhEYjL1dz2CSUQnUyipuCiucSFoSU5LCaoHFvMj6ZVnDSaMbB?cluster=devnet) |
| Deployment source commit          | `e01a2be705d640aa71dadcffd9c5115dc0db71dc`                                                                                                                                                                                           |
| Deployment source tree            | `e89b926a996a0c95633117d5cc9f82733903def1`                                                                                                                                                                                           |
| Upstream baseline                 | `81adb66daa1405eb1568af8b74f5c30924655bd6`                                                                                                                                                                                           |
| `Cargo.lock` SHA-256              | `3d508b3ad4e5ded45b79d24cdaf65b6befd8f1911024444ba71b2173df1e7bef`                                                                                                                                                                   |
| ELF length                        | `436360` bytes                                                                                                                                                                                                                       |
| ELF SHA-256                       | `fb64746885e19cd5a8a1f4f40c8a6dfff8183cc3d8e30b40a120a1d4dee7eb49`                                                                                                                                                                   |
| Pre-deployment CI                 | [GitHub Actions run 32080276502](https://github.com/quiknode-labs/token-wrap/actions/runs/32080276502)                                                                                                                               |

The transaction finalized without error. Finalized loader readback showed an executable Program account pointing to the ProgramData above, the non-null upgrade authority above, and an executable data region of exactly 436,360 bytes. `solana program dump` was byte-identical to the approved ELF. The named staging buffer was consumed by the successful initial deployment and no longer exists.

The program was not deployed with `--final`, its authority was not removed, and neither Program nor ProgramData was closed. Future changes must use the literal pWrap public address and the upgrade procedure in [`DEPLOYMENT.md`](./DEPLOYMENT.md); the program-account keypair is never used again for an upgrade.

## Initialization boundary

pWrap has no singleton initialization instruction or global configuration account. Loader
deployment made the instruction processor available immediately. Protocol state is created per
underlying/wrapped-mint relationship.

K256's selected existing-asset Devnet relationship has not been initialized. The exact mint gate,
required accounts, funding boundary, and lifecycle proof are defined in
[`DEVNET_EXISTING_ASSET_TEST.md`](./DEVNET_EXISTING_ASSET_TEST.md).

## Remaining gates

- This pWrap binary is distinct from the audited upstream snapshots and has not received its own security audit. It must not hold real value.
- The selected existing-asset confidential lifecycle has not yet been executed or proven.
- Independent sealed recovery copies of both durable authority keys must be created and restore-verified outside the repository before any future upgrade ceremony. No secret or recovery location belongs in this public record.
- A raw upgrade-authority signer can technically close or finalize the ProgramData. The permanent, upgradeable, never-close requirement remains an operator/governance invariant until authority moves to a controller that cannot authorize loader closure.
- Testnet and Mainnet were not touched by this Devnet ceremony. Mainnet was deployed later under a
  separate authorization and is recorded in [`MAINNET_DEPLOYMENT.md`](./MAINNET_DEPLOYMENT.md);
  Testnet remains undeployed.
