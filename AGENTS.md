# Fork Development Instructions

These instructions apply to the entire repository.

## Formatting Policy

The upstream repository is not clean under its current `.rustfmt.toml`: a
workspace-wide format rewrites many unrelated files. Keep downstream and
upstream PR diffs narrowly scoped.

- Never run `cargo fmt`, `cargo fmt --all`, or package-wide `cargo fmt` by
  default.
- Format only Rust files changed by the current work with
  `script/fmt-changed`.
- Check changed Rust files with `script/fmt-changed --check`.
- Use `script/fmt-changed --base <revision>` only when intentionally checking
  every Rust file changed since that revision.
- Inspect `git diff --stat` and `git diff` after formatting. Remove unrelated
  formatting hunks before committing.
- Do not mix a formatting baseline with a feature or bug-fix commit.
- Run a workspace-wide format only when the user explicitly requests it, and
  keep it in a dedicated commit.

The script uses the repository `.rustfmt.toml` and prevents rustfmt from
recursively formatting child modules that were not passed explicitly.

## Upstream PRs

- Keep each upstream PR focused on one behavior.
- Create PR branches from `upstream/main`, not from `downstream/rusq`.
- Cherry-pick only the relevant downstream commits.
- Verify that the PR diff contains no unrelated formatting changes.
