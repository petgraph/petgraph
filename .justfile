#!/usr/bin/env just --justfile

set dotenv-load := true

repo := `git rev-parse --show-toplevel`
profile := env_var_or_default('PROFILE', "dev")
github-event-name := env_var_or_default('GITHUB_EVENT_NAME', "none")


######################################################################
## Helper to print a message when calling `just`
######################################################################
[private]
default:
  @echo "Usage: just <recipe>"
  @just list-repo-recipes
  @echo "For further information, run 'just --help'"

# List recipes in this file and from the calling directory
[private]
usage:
  @echo "Usage: just <recipe>"
  @just list-recipes
  @echo "For further information, run 'just --help'"

[private]
list-recipes:
  @echo "\nRepository recipes:"
  @just --list --unsorted --list-heading ''
