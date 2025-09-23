---
name: New Algorithm
about: Offer a new algorithm to be added to petgraph!
title: ''
labels: A-crate, C-new-algorithm, S-Needs-Triage
assignees: ''
---

<!--
  -- Thanks for filing a `petgraph` issue!
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

### What algorithm are you requesting to be added?

If this addresses an existing feature request, please just add a link to that issue, with "Resolves #ISSUE_NUMBER".

Otherwise please add a description of the algorithm to be added. Try to answer the following questions:

- What (graph) problem does it solve?
- What is the basic idea of the algorithm?
- What other, reference implementations exist / did you use?

### Please make sure to check the following:

[ ] Added a `regular` test for the new algorithm in the `tests/` directory.

[ ] Added a `quickcheck` property test for the new algorithm in the `tests/quickcheck.rs` file.

[ ] Added a `benchmark` test for measuring performance of the new algorithm

[ ] Made the algorithm work with generic graphs, constraining the
  generic graph type parameter with our existing graph traits, like ``Visitable``,
  or with new graph traits

[ ] Wrote a docstring for the algorithm, according to the algorithm
  documentation guidelines in `assets/guide/algodocs.md`