# Numi

`numi` is a crate for primitives with a focus on numerical traits, akin to existing implementations such
as [`num-traits`].
The current goal of `numi` is to be a companion crate to [`num-traits`], instead of a replacement.

Libraries should strive to implement these traits on their own, but as a convenience feature for early adopters default
implementations have been provided. If you are a library author and would like to implement these traits, please open an
issue so that the default implementations can be removed.

The home of numi is yet to be decided until publication but will be

[`num-traits`]: https://crates.io/crates/num-traits
