---
name: Bug Fix
about: You fixed a bug? Great!
title: ''
labels: A-crate, B-uncategorized, S-Needs-Triage
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

### What bug did you fix?

If this addresses an existing issue, please just add a link to that issue, with "Resolves #ISSUE_NUMBER".

Otherwise, describe the bug that you fixed. Try to answer the following questions:

- What was the unexpected behavior?
- Under which conditions did it occur?
- How did you fix it?

### Please make sure to check the following

[] Added a regression test for the bug so that we can be sure that we never accidentally re-introduce the bug again

[] If the PR contains a breaking change, explain the change and why it is necessary
