# mapbox-gl-native-rs-public

## Motivation

A Rust re-implementation to https://github.com/mapbox/mapbox-gl-native. 

Please note that some proprietary code has been removed, including some internal request endpoint URL, headers, tokens etc. So it should compile but not executable.

Tony wants to:
- Learn Rust
- Learn about how mapbox-gl-native works

## Setup

- MacOS 10.14.6
  - `brew install sdl2 cmake`

## Technical Overview

Threading

## Hints
- seeing "Blocking waiting for file lock on package cache" ?
  - Run this: `rm -rf ~/.cargo/registry/index/*`

## References

MapBox Stylesheet Spec: https://docs.mapbox.com/mapbox-gl-js/style-spec/

Crate gfx docs: `cargo doc -p gfx --no-deps --open`

## Contact

Tony loves to hear about your feedback: [healthytony@gmail.com](mailto:healthytony@gmail.com)
