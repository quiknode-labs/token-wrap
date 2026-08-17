# pWrap JavaScript client

Generated TypeScript helpers for the k256-maintained pWrap program at
`pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2`.

This package is private and is not published to npm. Build and consume it from this
repository until a separate public-client release is reviewed and approved.

## Verify

From the repository root:

```bash
pnpm --dir clients/js test
pnpm --dir clients/js build
pnpm --dir clients/js lint
pnpm --dir clients/js format:check
```

Regenerate the client only after changing `program/idl.ts`:

```bash
pnpm generate:clients
```
