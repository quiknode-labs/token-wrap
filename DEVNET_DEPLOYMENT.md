# pWrap devnet deployment record

> **Retired historical fixture:** The synthetic asset in this record is not a supported K256 asset
> and must not be recreated, funded, sponsored, canonicalized, or reused. The addresses and
> transactions remain here only as immutable evidence of the initial program and protocol test.
> Current work accepts only exact underlying mints that already exist independently of K256; see
> [`DEVNET_EXISTING_ASSET_TEST.md`](./DEVNET_EXISTING_ASSET_TEST.md).

This is the public evidence record for the permanent pWrap devnet address. It records a valueless protocol experiment, not approval for real-value assets, testnet, or mainnet.

## Live program

| Item                        | Finalized value                                                                                                                                                                                                                      |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Cluster                     | Solana devnet                                                                                                                                                                                                                        |
| Genesis hash                | `EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG`                                                                                                                                                                                       |
| Program                     | [`pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2`](https://explorer.solana.com/address/pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2?cluster=devnet)                                                                                      |
| Loader                      | `BPFLoaderUpgradeab1e11111111111111111111111`                                                                                                                                                                                        |
| ProgramData                 | `DJ7bADfr6LxWQsyzRGJXRwieXB4CmkRioTLpMoPvMhW1`                                                                                                                                                                                       |
| Upgrade authority           | `G1kdS4CCCKZFKzupAm9N5ZLMvx5bgfzpUx9xkmt1KxYR`                                                                                                                                                                                       |
| Initial deploy slot         | `484920383`                                                                                                                                                                                                                          |
| Initial deploy time         | `2026-08-17T23:28:51Z`                                                                                                                                                                                                               |
| Post-test verification slot | `484925035`                                                                                                                                                                                                                          |
| Initial deploy transaction  | [`3czLqqm16e1wjdg89RoaKTKQk2cniKGXrL5rtefEhEYjL1dz2CSUQnUyipuCiucSFoSU5LCaoHFvMj6ZVnDSaMbB`](https://explorer.solana.com/tx/3czLqqm16e1wjdg89RoaKTKQk2cniKGXrL5rtefEhEYjL1dz2CSUQnUyipuCiucSFoSU5LCaoHFvMj6ZVnDSaMbB?cluster=devnet) |
| Deployment source commit    | `e01a2be705d640aa71dadcffd9c5115dc0db71dc`                                                                                                                                                                                           |
| Deployment source tree      | `e89b926a996a0c95633117d5cc9f82733903def1`                                                                                                                                                                                           |
| Upstream baseline           | `81adb66daa1405eb1568af8b74f5c30924655bd6`                                                                                                                                                                                           |
| `Cargo.lock` SHA-256        | `3d508b3ad4e5ded45b79d24cdaf65b6befd8f1911024444ba71b2173df1e7bef`                                                                                                                                                                   |
| ELF length                  | `436360` bytes                                                                                                                                                                                                                       |
| ELF SHA-256                 | `fb64746885e19cd5a8a1f4f40c8a6dfff8183cc3d8e30b40a120a1d4dee7eb49`                                                                                                                                                                   |
| Pre-deployment CI           | [GitHub Actions run 32080276502](https://github.com/quiknode-labs/token-wrap/actions/runs/32080276502)                                                                                                                               |

The transaction finalized without error. Finalized loader readback showed an executable Program account pointing to the ProgramData above, the non-null upgrade authority above, and an executable data region of exactly 436,360 bytes. `solana program dump` was byte-identical to the approved ELF. The named staging buffer was consumed by the successful initial deployment and no longer exists.

The program was not deployed with `--final`, its authority was not removed, and neither Program nor ProgramData was closed. Future changes must use the literal pWrap public address and the upgrade procedure in [`DEPLOYMENT.md`](./DEPLOYMENT.md); the program-account keypair is never used again for an upgrade.

## Initialization result

pWrap has no singleton initialization instruction or global configuration account. Loader deployment made the instruction processor available immediately. All protocol state is created per underlying/wrapped-mint relationship.

The first relationship is the intentionally valueless KTEST fixture:

| Account                    | Address and state                                                                                                            |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Underlying KTEST mint      | `ALMLnHwuN23CkVVXwmQcaBj6ijEuhsQiDyE4Ni2xYhuJ`; legacy SPL Token; 6 decimals; mint authority `None`; freeze authority `None` |
| Wrapped KTEST mint         | `FjXMPwkGvWENVxHK1NcwW6UMyk5Q4FTcPu3MXSsehegW`; Token-2022; 6 decimals; pWrap PDA mint authority; freeze authority `None`    |
| Wrapped-mint authority PDA | `4Shyut6veB1UGwPsSm8Ua7wteBAUviVJcAydNxY5Ma3C`                                                                               |
| Backpointer PDA            | `5ms4K8SYuq3TJ6g5YamAv3B91LskgHgFzeSFDX5sUtke`; decodes to the KTEST underlying mint                                         |
| Canonical-pointer PDA      | `533inSSg7hgAiZSJkfPsDeYo9rLR7Nch2M5SavyTt8KD`; decodes to the pWrap program ID                                              |
| Underlying escrow          | `8pNFGmkFN4xb3gHXZsfZCUCFBo2JFkyLZawWU9UPi8HD`; legacy SPL Token ATA owned by the wrapped-mint authority PDA                 |
| Alice underlying account   | `G1JtR7174jpkksC2qeZczXtWDzbK3vBvXyyqrsDrtDW1`                                                                               |
| Alice wrapped account      | `7X18HqAV4g3UYt76HzXzPvP4nGiaKpukri6SeDYDtLsF`                                                                               |
| Bob underlying account     | `3P5YFdnyrkBZbTx7226um7t8NL6U8EAsvwQAJdvoVTTJ`                                                                               |
| Bob wrapped account        | `b747c5RdRMWQKi5Uf6hc59rsz45LozT5qvVZYYp8ADh`                                                                                |

The wrapped mint's immutable confidential-transfer configuration has automatic account approval, no configuration authority, and no auditor. The canonical pointer was written before the KTEST mint authority was permanently revoked.

## Finalized lifecycle proof

The fixture minted exactly `1_000_000` raw units, or one KTEST, and used the Agave v4.1.0 bundled `spl-token-cli 5.6.1` for Token-2022 confidential operations. Every transaction below was read back at finalized commitment with `err: null`; a post-test batch read confirmed all 20 deployment and lifecycle transactions remained finalized and clean.

| Phase                                | Finalized transaction                                                                                                                                                                                                                |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Create underlying mint               | [`5wRoJKYy5WJBfxvz5FagGPVZwynbqkFgJ29dadXo4Aa3hjdyLYcKK53XeppLoGCNHyr4crNGVGje6mkMEorA6QbU`](https://explorer.solana.com/tx/5wRoJKYy5WJBfxvz5FagGPVZwynbqkFgJ29dadXo4Aa3hjdyLYcKK53XeppLoGCNHyr4crNGVGje6mkMEorA6QbU?cluster=devnet) |
| Create Alice underlying account      | [`3Q5xxHYivzHW2H1MdcXVR53znTYhSpyJpqdzq82CFQwwoAs3v6zZSZvtzvbz2YgAyARNiAMY5MxQ5uv3t572pTjt`](https://explorer.solana.com/tx/3Q5xxHYivzHW2H1MdcXVR53znTYhSpyJpqdzq82CFQwwoAs3v6zZSZvtzvbz2YgAyARNiAMY5MxQ5uv3t572pTjt?cluster=devnet) |
| Create Bob underlying account        | [`5r8Jaay4nJxEFNMbQVcatVw11y8cgqcvEUq5iYdf64HHo5iYmoyGQ1iaYtxYkwbGwLfkD8QsrYZUoeTD3TvR4Zc3`](https://explorer.solana.com/tx/5r8Jaay4nJxEFNMbQVcatVw11y8cgqcvEUq5iYdf64HHo5iYmoyGQ1iaYtxYkwbGwLfkD8QsrYZUoeTD3TvR4Zc3?cluster=devnet) |
| Mint one KTEST                       | [`nBy5dL6axtpYYxT3gg2ufbbvwZfXkcm52Q5MB2zjrSBoZ6x6LEFBWF4ruo3tUiHXQS887BBvTBJ2bkh8h9271Vn`](https://explorer.solana.com/tx/nBy5dL6axtpYYxT3gg2ufbbvwZfXkcm52Q5MB2zjrSBoZ6x6LEFBWF4ruo3tUiHXQS887BBvTBJ2bkh8h9271Vn?cluster=devnet)   |
| pWrap `CreateMint`                   | [`5LyJDPhGjY89tQxgGZoA7jESHCxdqBkny5tzsqs8BbZ8aFALfPPU1ncCREFfY6B4V1UwqtTyjDs3n2ApuxuUuZVR`](https://explorer.solana.com/tx/5LyJDPhGjY89tQxgGZoA7jESHCxdqBkny5tzsqs8BbZ8aFALfPPU1ncCREFfY6B4V1UwqtTyjDs3n2ApuxuUuZVR?cluster=devnet) |
| Create escrow                        | [`2j9YJoD5mKH7zi2BRzgNyE716YPSz6B1XkjFbjhQ1B8F3kC1pU15LUEYUSXLLihzTQH7eAe6UEFDZhmS1ThENrxT`](https://explorer.solana.com/tx/2j9YJoD5mKH7zi2BRzgNyE716YPSz6B1XkjFbjhQ1B8F3kC1pU15LUEYUSXLLihzTQH7eAe6UEFDZhmS1ThENrxT?cluster=devnet) |
| Set canonical pointer                | [`utd7ndNKTiyLLQ7HAbCe3XYf1H3c2ac5y6duQra3ioNQ3kEBWgrNRu92qzg5FbZSgvqZhoaasCAztpNpcVEaDxm`](https://explorer.solana.com/tx/utd7ndNKTiyLLQ7HAbCe3XYf1H3c2ac5y6duQra3ioNQ3kEBWgrNRu92qzg5FbZSgvqZhoaasCAztpNpcVEaDxm?cluster=devnet)   |
| Revoke KTEST mint authority          | [`37jt3ufFFDTdRQzVW6Zrb3tAiKXMhGoVfC6tsa93jm6aebw8fpxJtw1KLKiKbwmC8CyYC7zt1fvbGTLEgMKSEwEY`](https://explorer.solana.com/tx/37jt3ufFFDTdRQzVW6Zrb3tAiKXMhGoVfC6tsa93jm6aebw8fpxJtw1KLKiKbwmC8CyYC7zt1fvbGTLEgMKSEwEY?cluster=devnet) |
| Create Alice wrapped account         | [`4jJPG48VVeuGEhAVPdpdYsbS8QL7Y3efazEis29uv4QEU7vUiY3JT1wy8ZEM6agbpLtKGSqYLbBNDoyJqw53PWZc`](https://explorer.solana.com/tx/4jJPG48VVeuGEhAVPdpdYsbS8QL7Y3efazEis29uv4QEU7vUiY3JT1wy8ZEM6agbpLtKGSqYLbBNDoyJqw53PWZc?cluster=devnet) |
| Create Bob wrapped account           | [`tS2ykbz7T9hKJZ8oi6Q2UB6VjBuBWGLxprcR3VY7495abDYK8XrTnumeK5Pnmg4A695ZmYxUfw6M8SGwS7nbV9o`](https://explorer.solana.com/tx/tS2ykbz7T9hKJZ8oi6Q2UB6VjBuBWGLxprcR3VY7495abDYK8XrTnumeK5Pnmg4A695ZmYxUfw6M8SGwS7nbV9o?cluster=devnet)   |
| Configure Alice confidential account | [`2Ghh3GH9YsUze2VaZCndK4TAXv7Q4muPDPXriHaJDhW7V33QWhnChJg3Lb4BwpPyz1PYB7g2LxsBMD4v35axeUsC`](https://explorer.solana.com/tx/2Ghh3GH9YsUze2VaZCndK4TAXv7Q4muPDPXriHaJDhW7V33QWhnChJg3Lb4BwpPyz1PYB7g2LxsBMD4v35axeUsC?cluster=devnet) |
| Configure Bob confidential account   | [`5A8nXTPtwUrW1D9q4o68aBdSC1Vzn9W8SqjrvkWY8jJzi4jCV9Jrd7RMW3QMdaDn4sj9YEyd87ScDRg3Evzf9QX3`](https://explorer.solana.com/tx/5A8nXTPtwUrW1D9q4o68aBdSC1Vzn9W8SqjrvkWY8jJzi4jCV9Jrd7RMW3QMdaDn4sj9YEyd87ScDRg3Evzf9QX3?cluster=devnet) |
| pWrap full public wrap               | [`3VQMfDqqjb1z6qFQphQLKC2G8Hae4TVF8rW183AbwGbbo937YGVg37MpvssM3N2nQTj61xPh5cgWB5WtYQPh2bor`](https://explorer.solana.com/tx/3VQMfDqqjb1z6qFQphQLKC2G8Hae4TVF8rW183AbwGbbo937YGVg37MpvssM3N2nQTj61xPh5cgWB5WtYQPh2bor?cluster=devnet) |
| Deposit Alice public balance         | [`5h6RegPMmQJDXCTDQgc9NQfunYeWsfHXhs4McxwzAkrUg4ScrQ6U13zJSH47it6AWv8LXY1wDXpyiNJdpydzTYY5`](https://explorer.solana.com/tx/5h6RegPMmQJDXCTDQgc9NQfunYeWsfHXhs4McxwzAkrUg4ScrQ6U13zJSH47it6AWv8LXY1wDXpyiNJdpydzTYY5?cluster=devnet) |
| Apply Alice pending balance          | [`3Gvkyqd6pBSoQRwvdRjUCxHxfHwvQt4tM2khs5iEmDnAQs21RXVhFPJHoTArN1NWV1ct5WuGiEu5FaB8J51r6jyJ`](https://explorer.solana.com/tx/3Gvkyqd6pBSoQRwvdRjUCxHxfHwvQt4tM2khs5iEmDnAQs21RXVhFPJHoTArN1NWV1ct5WuGiEu5FaB8J51r6jyJ?cluster=devnet) |
| Confidential transfer Alice to Bob   | [`4E6Z2LwMGoL8pFq87e97T4ycXMjB2Bwd1BJhMYB6N3u3yhFNzRYTwvDZb3C1ukiwAnz16LHq68oLrqTcVk9wsUi6`](https://explorer.solana.com/tx/4E6Z2LwMGoL8pFq87e97T4ycXMjB2Bwd1BJhMYB6N3u3yhFNzRYTwvDZb3C1ukiwAnz16LHq68oLrqTcVk9wsUi6?cluster=devnet) |
| Apply Bob pending balance            | [`5zVeQoRQrm3azmT2KEytorGFAy4RRKY44MB3pCs1Jhb9LHXBG5djcuuzn6hsYDSqpZSSrgM1pa67ujV7fPh39oWb`](https://explorer.solana.com/tx/5zVeQoRQrm3azmT2KEytorGFAy4RRKY44MB3pCs1Jhb9LHXBG5djcuuzn6hsYDSqpZSSrgM1pa67ujV7fPh39oWb?cluster=devnet) |
| Withdraw Bob confidential balance    | [`299Re8TB41wKAHAwCmbkpkFoqS75WS3ZK6njrxLsWL4pM93tedsYBrJtLp1ndvgw7qDSyHJAbLr7RbAAddyRj8m9`](https://explorer.solana.com/tx/299Re8TB41wKAHAwCmbkpkFoqS75WS3ZK6njrxLsWL4pM93tedsYBrJtLp1ndvgw7qDSyHJAbLr7RbAAddyRj8m9?cluster=devnet) |
| pWrap full public unwrap             | [`3Z8z3mZnd5jxj77Haesxk9oU924ApmEPMzf9yFGUzxXmGka5qnQGpSFce54BHce1iLhf5Br9VvntWt9gxGR9SQhG`](https://explorer.solana.com/tx/3Z8z3mZnd5jxj77Haesxk9oU924ApmEPMzf9yFGUzxXmGka5qnQGpSFce54BHce1iLhf5Br9VvntWt9gxGR9SQhG?cluster=devnet) |

Observed state transitions:

1. Public wrap moved one KTEST from Alice to escrow and minted one public wrapped KTEST.
2. Deposit and apply moved the public wrapped balance into Alice's confidential available balance.
3. Confidential transfer and apply moved the full balance to Bob without changing escrow or wrapped supply.
4. Withdraw moved Bob's confidential balance back to public wrapped balance.
5. Public unwrap burned the full wrapped balance and released the escrowed KTEST to Bob.

Final state is Bob underlying `1_000_000`, escrow `0`, wrapped supply `0`, and both public wrapped accounts `0`. A zero wrapped supply also proves no confidential wrapped balance remains.

## Negative simulation proof

The CLI's `--dry-run` path was corrected after deployment to fail closed on `simulateTransaction.value.err` and include the returned logs. QuickNode devnet simulations against the live program then produced nonzero exits for:

- wrap amount `0`: pWrap `ZeroWrapAmount`, custom error `0x2`;
- wrap `1_000_001` from Bob's `1_000_000` account: SPL Token `insufficient funds`, custom error `0x1`;
- unwrap `1` from Bob's zero public wrapped balance: Token-2022 `insufficient funds`, custom error `0x1`.

These were simulations only. They were not submitted, produced no on-chain transaction signature, and changed no account state. The live ELF was dumped again afterward and remained byte-identical to the deployment artifact; ProgramData and upgrade authority were unchanged.

## Remaining gates

- This pWrap binary is distinct from the audited upstream snapshots and has not received its own security audit. It must not hold real value.
- The successful confidential lifecycle proves the on-chain protocol with the pinned CLI, not the intended browser-wallet signing and recovery flow.
- Independent sealed recovery copies of both durable authority keys must be created and restore-verified outside the repository before any future upgrade ceremony. No secret or recovery location belongs in this public record.
- A raw upgrade-authority signer can technically close or finalize the ProgramData. The permanent, upgradeable, never-close requirement remains an operator/governance invariant until authority moves to a controller that cannot authorize loader closure.
- Testnet and Mainnet were not touched by this Devnet ceremony. Mainnet was deployed later under a
  separate authorization and is recorded in [`MAINNET_DEPLOYMENT.md`](./MAINNET_DEPLOYMENT.md);
  Testnet remains undeployed.
