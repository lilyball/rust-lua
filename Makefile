include common.mk

.PHONY: test all clean

all: $(LIBNAME)

$(LIBNAME): $(filter-out tests.rs,$(wildcard *.rs))
	rustc lib.rs

test: $(wildcard *.rs)
	rustc --test lib.rs
	env RUST_THREADS=1 ./lua $(TESTNAME)

clean:
	rm -f lua $(LIBNAME)
