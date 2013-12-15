include common.mk

.PHONY: test all clean examples

all: $(LIBNAME)

$(LIBNAME): $(filter-out tests.rs,$(wildcard *.rs))
	rustc lib.rs

$(LIBNAME): config.rs

config.rs: gen-config
	./gen-config > $@

.INTERMEDIATE: gen-config
gen-config: config.c
	clang -o $@ $<

test: $(wildcard *.rs) config.rs
	rustc --test lib.rs
	env RUST_THREADS=1 ./lua $(TESTNAME)

clean:
	rm -f lua $(LIBNAME) config.rs
	$(MAKE) -C examples clean

examples:
	$(MAKE) -C examples
