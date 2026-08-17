#!/usr/bin/env bash

set -euo pipefail

expected_program_id="pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2"

if [[ $# -ne 1 ]]; then
  echo "usage: $0 /path/to/program-keypair.json" >&2
  exit 2
fi

keypair_path=$1
if [[ ! -f "$keypair_path" ]]; then
  echo "program keypair does not exist: $keypair_path" >&2
  exit 1
fi

if ! command -v solana-keygen >/dev/null 2>&1; then
  echo "solana-keygen is required" >&2
  exit 1
fi

actual_program_id=$(solana-keygen pubkey "$keypair_path")
if [[ "$actual_program_id" != "$expected_program_id" ]]; then
  echo "program keypair derives $actual_program_id, expected $expected_program_id" >&2
  exit 1
fi

permissions=$(stat -f '%OLp' "$keypair_path" 2>/dev/null || stat -c '%a' "$keypair_path")
if [[ "${permissions: -2}" != "00" ]]; then
  echo "program keypair permissions are $permissions; remove all group/other access" >&2
  exit 1
fi

keypair_directory=$(cd "$(dirname "$keypair_path")" && pwd)
keypair_absolute=$keypair_directory/$(basename "$keypair_path")

# Resolve the repository that contains the key, rather than the repository from
# which this script was invoked. The intended key lives next to this nested fork
# in the enclosing monorepo.
if keypair_repo_root=$(git -C "$keypair_directory" rev-parse --show-toplevel 2>/dev/null); then
  case "$keypair_absolute" in
    "$keypair_repo_root"/*)
      keypair_relative=${keypair_absolute#"$keypair_repo_root"/}
      if git -C "$keypair_repo_root" ls-files --error-unmatch "$keypair_relative" >/dev/null 2>&1; then
        echo "program keypair is tracked by git; remove it from the index and repository history" >&2
        exit 1
      fi

      keypair_history=$(git -C "$keypair_repo_root" log --all --format='%H' -- "$keypair_relative")
      if [[ -n "$keypair_history" ]]; then
        echo "program keypair exists in git history; rotate it before deployment" >&2
        exit 1
      fi
      ;;
  esac
fi

echo "$actual_program_id"
