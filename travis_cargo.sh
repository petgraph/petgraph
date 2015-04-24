#!/bin/sh

# build with all feature flags in the nightly

FEATURES=

if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then
    FEATURES="--features test"
else
    if [ "$1" = "bench" ]; then exit 0; fi
fi

exec cargo "$@" $FEATURES
