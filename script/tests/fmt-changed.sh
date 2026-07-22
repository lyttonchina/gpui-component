#!/usr/bin/env bash

set -euo pipefail

repository_root=$(cd "$(dirname "$0")/../.." && pwd)
subject="$repository_root/script/fmt-changed"
test_root=$(mktemp -d)
trap 'rm -rf "$test_root"' EXIT

mkdir -p "$test_root/bin" "$test_root/repository"

cat >"$test_root/bin/rustfmt" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$@" >"$RUSTFMT_LOG"
EOF
chmod +x "$test_root/bin/rustfmt"

cd "$test_root/repository"
git init -q
git config user.name "fmt-changed test"
git config user.email "fmt-changed@example.com"

printf 'fn changed() {}\n' >changed.rs
printf 'fn untouched() {}\n' >untouched.rs
printf 'original\n' >notes.txt
git add .
git commit -qm "initial"

printf 'fn changed( ) { }\n' >changed.rs
printf 'updated\n' >notes.txt
printf 'fn added( ) { }\n' >added.rs

RUSTFMT_LOG="$test_root/rustfmt.log" PATH="$test_root/bin:$PATH" "$subject" --check

grep -Fx -- '--check' "$test_root/rustfmt.log" >/dev/null
grep -Fx -- 'changed.rs' "$test_root/rustfmt.log" >/dev/null
grep -Fx -- 'added.rs' "$test_root/rustfmt.log" >/dev/null

if grep -Fx -- 'untouched.rs' "$test_root/rustfmt.log" >/dev/null; then
  echo "fmt-changed passed an untouched Rust file to rustfmt" >&2
  exit 1
fi

if grep -Fx -- 'notes.txt' "$test_root/rustfmt.log" >/dev/null; then
  echo "fmt-changed passed a non-Rust file to rustfmt" >&2
  exit 1
fi

echo "fmt-changed test passed"
