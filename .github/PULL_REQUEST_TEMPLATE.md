---
labels: S-Needs-Triage
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
  -->

<!-- From the templates below, please select the one that fits your PR and remove the other parts. We appreciate it 🦕 -->

## 🐛 Bug Fix

### What bug did you fix?

If this addresses an existing issue, please just add a link to that issue, with "Resolves #ISSUE_NUMBER".

Otherwise, describe the bug that you fixed. Try to answer the following questions:

- What was the unexpected behavior?
- Under which conditions did it occur?
- How did you fix it?

### Please make sure to check the following

- [] Added a regression test for the bug so that we can be sure that we never accidentally re-introduce the bug again
- [] If the PR contains a breaking change, explain the change and why it is necessary



## ✨ New Algorithm

### What algorithm are you requesting to be added?

If this addresses an existing feature request, please just add a link to that issue, with "Resolves #ISSUE_NUMBER".

Otherwise please add a description of the algorithm to be added. Try to answer the following questions:

- What (graph) problem does it solve?
- What is the basic idea of the algorithm?
- What other, reference implementations exist / did you use?

### Please make sure to check the following

- [] Added a regular test for the new algorithm in the [`tests/`](https://github.com/petgraph/petgraph/tree/master/tests) directory.
- [] Added a `quickcheck` property test for the new algorithm in the [`tests/quickcheck.rs`](https://github.com/petgraph/petgraph/blob/master/tests/quickcheck.rs) file.
- [] Added a `benchmark` test for measuring performance of the new algorithm
- [] Made the algorithm work with generic graphs, constraining the
  generic graph type parameter with our existing graph traits, like ``Visitable``,
  or with new graph traits
- [] Wrote a docstring for the algorithm, according to the algorithm
  documentation guidelines in [assets/guide/algodocs.md](https://github.com/petgraph/petgraph/blob/master/assets/guide/algodocs.md)
- [] If the PR contains a breaking change, explained the change and why it is necessary



## ⚡️ Performance Improvement

### What part of petgraph did you improve the performance of?

Describe which algorithm or part of the crate you improved the performance of.

### Please make sure to check the following

- [] Added a new `#[bench]` function that exercises this code path, if one doesn't already exist
- [] Added before and after `cargo bench` scores, optionally formatted using [`cargo-benchcmp`](https://github.com/BurntSushi/cargo-benchcmp) to this PR.
- [] If the PR contains a breaking change, explained the change and why it is necessary



## 🔧 Other Change

### What change are you making to petgraph?

If this addresses an existing issue, please just add a link to that issue, with "Resolves #ISSUE_NUMBER".

### Please make sure to check the following

- [] If the PR contains a breaking change, explain the change and why it is necessary