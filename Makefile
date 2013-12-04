.PHONY: test all

all: $(filter-out tests.rs,$(wildcard *.rs))
	rustc lib.rs

test: $(wildcard *.rs)
	rustc --test lib.rs
	env RUST_THREADS=1 ./lua
