# mapbox-gl-native-rs

## Motivation

A Rust re-implementation to https://github.com/mapbox/mapbox-gl-native

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