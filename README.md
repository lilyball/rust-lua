# rust-lua

[![Build Status](https://travis-ci.org/lilyball/rust-lua.svg)](https://travis-ci.org/lilyball/rust-lua)

Copyright 2014 Lily Ballard

## Description

This is a set of Rust bindings to Lua 5.1.

The goal is to provide a (relatively) safe interface to Lua that closely
mirrors its C API.

The bindings are complete, but largely untested. Every non-unsafe function
does its best to enforce safety. Lua 5.1.5 was used as a reference for
internal implementation details that affect safety (e.g. stack space needed
for auxlib functions).

Unfortunately, there are very few tests. It turns out to be complicated to
properly test the Lua C API, and there are very few examples that could be
used as tests. Please let me know if there are any bugs.

## Installation

    make all

To run the tests, use the test make target:

    make test
