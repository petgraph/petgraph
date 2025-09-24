---
name: Performance Improvement
about: You made an algorithm or some other crate functionality faster? Awesome!
title: ''
labels: A-crate, C-performance, S-Needs-Triage
assignees: ''
---

<!--
  -- Thanks for opening a `petgraph` pull request!
  -- We require PR titles to follow the Conventional Commits specification,
  -- https://www.conventionalcommits.org/en/v1.0.0/. This helps us generate
  -- changelogs and follow semantic versioning.
  --
  -- Start the PR title with one of the following:
  --  * `feat:` for new features
  --  * `fix:` for bug fixes
  --  * `refactor:` for code refactors
  --  * `docs:` for documentation changes
  --  * `test:` for test changes
  --  * `perf:` for performance improvements
  --  * `revert:` for reverting changes
  --  * `ci:` for CI/CD changes
  --  * `chore:` for changes that don't fit in any of the above categories
  -- The last two categories will not be included in the changelog.
  --
  -- If your PR includes a breaking change, please add a `!` after the type
  -- and include a `BREAKING CHANGE:` line in the body of the PR describing
  -- the necessary changes for users to update their code.
  --
  -- Please also fill out the template below. We appreciate it :)
  -->

### What part of petgraph did you improve the performance of?

Describe which algorithm or part of the crate you improved the performance of.

### Please make sure to check the following

[] Added a new `#[bench]` function that exercises this code path, if one doesn't already exist

[] Added before and after `cargo bench` scores, optionally formatted using [`cargo-benchcmp`](https://github.com/BurntSushi/cargo-benchcmp) to this PR.

[] If the PR contains a breaking change, explained the change and why it is necessary
