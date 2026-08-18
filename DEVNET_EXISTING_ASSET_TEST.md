# Devnet existing-asset test plan

This is the only active asset plan for the deployed pWrap program. K256 does not create an
underlying mint for this test.

## Selected asset

| Item                   | Value                                                           |
| ---------------------- | --------------------------------------------------------------- |
| Cluster                | Solana Devnet                                                   |
| pWrap                  | `pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2`                   |
| Asset                  | Circle-published Devnet test USDC                               |
| Underlying mint        | `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`                  |
| Token program          | Legacy SPL Token, `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` |
| Decimals               | `6`                                                             |
| Wrapped W              | `AEQhvBtU414zr8Agcg7gyUBDTrqcnZaxuNLKgruMS4KC`                  |
| Wrapped-mint authority | `5Nqq1VLd3ni6rZLT2QzoaRMtCwpWrVemRwoFogs27117`                  |
| Backpointer            | `EQodrD4rnN2iaVML5381eKG6GR34joREK4XNEHr4KRqB`                  |
| Escrow                 | `AfxMDkmxDG4M1ZBoKxA2NGu829XKzQL88bmXJ5M7TmVz`                  |
| Current status         | Funding pending; all three relationship accounts absent         |

Circle publishes the mint in its
[contract-address registry](https://developers.circle.com/stablecoins/usdc-contract-addresses).
This is a valueless issuer-published test token, not dollar-backed mainnet USDC, and pWrap is not
Circle-endorsed.

No official Tether Solana Devnet USDT mint has been verified, so USDT is not allowed. Native SOL is
not a mint; canonical WSOL is deferred until its exact native-account path is separately proven.

## Funding request

| Wallet | Public owner                                   | Funding                                       |
| ------ | ---------------------------------------------- | --------------------------------------------- |
| Alice  | `GnWNHnvLhnvCzbGgcEgUjAyPKa99zva7HS9Q3uqqwfjX` | `2` Circle Devnet USDC plus `0.25` Devnet SOL |
| Bob    | `6i2GWcFncsXXi64TmPMHKGmS7J6whdUrNze6oYTHKRiU` | `0.25` Devnet SOL; no USDC required           |

Send to the owner addresses, not to the mint, pWrap, W, backpointer, or escrow. Alice's canonical
USDC ATA derives to `7n6hqMN8bv4Qb157G2yMAz7BHJ35NF4rHdMo7nVbtYkz`; Bob's derives to
`GhGW5McM598dNXxw3vuJBZbJFfSf93jqUmcGTjzLAh3o`. Both were absent at the recorded preflight.

The future Token-2022 W ATAs derive to
`H3CYaSDkzQwnBXBB2f8z7vEUStXEtW6waaK8zKg2C14f` for Alice and
`HBxf4NBdsuuPuK8V1Bxsrhi3XV64ohytC3AoW32mehVi` for Bob. They are also absent and must be created
after W exists, before Wrap or Confidential Transfer configuration.

Do not use the repository-tracked shared funder key. Alice is the disposable fee payer after
funding; durable pWrap authority remains separate.

## Execution gate

After funding is finalized:

1. Re-read Devnet genesis and the exact U mint owner, decimals, mint authority, and freeze authority.
2. Require executable Token-2022 `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` under the
   upgradeable loader and executable ZK ElGamal `ZkE1Gama1Proof11111111111111111111111111111`
   under the native loader. Require the ZK enable feature; stop when temporary disable is active
   without the later re-enable feature. Exact feature IDs: enable
   `zkhiy5oLowR7HY4zogXjCjeMXyruLqBwSWH21qcFtnv`, disable
   `zkdoVwnSFnSLtGJG7irJPEYUpmb4i7sGMGcnN6T9rnC`, re-enable
   `zkexuyPRdyTVbZqEAREueqL2xvvoBhRgth9xGSc1tMN`.
3. Re-read pWrap ProgramData, upgrade authority, and live ELF hash.
4. Re-derive W, backpointer, authority, and escrow; require absent-or-exact state.
5. Create W/backpointer, then the underlying escrow ATA. Do not create a canonical pointer because
   K256 does not control the currently observed underlying mint-authority key.
6. Create or verify both underlying-USDC ATAs, then create both Token-2022 W ATAs. Wrap and Unwrap
   require existing destination token accounts; `create-mint` does not create user accounts.
7. Configure Alice and Bob for Confidential Transfer while both W accounts are empty.
8. Wrap exactly `1_000_000` raw USDC units, then execute Deposit → Apply → confidential Transfer →
   Apply → Withdraw.
9. Unwrap Bob's public W to his existing underlying-USDC ATA.
10. Require finalized readback, zero escrow, zero W supply, zero confidential remainder, and exactly
    `1_000_000` raw underlying units delivered to Bob.
11. Prove zero, over-balance, empty, malformed relationship, wrong-program, and unauthorized-pointer
    paths fail without submission.

Every write requires a separate explicit authorization. This plan records readiness and funding
requirements; it does not authorize a transaction by itself.
