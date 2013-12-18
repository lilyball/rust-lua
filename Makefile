include common.mk

.PHONY: test all clean examples

LUA_PCNAME := $(if $(shell pkg-config --exists lua5.1 && echo yes),lua5.1,lua)
CFLAGS += $(shell pkg-config --cflags $(LUA_PCNAME))

all: $(LIBNAME)

$(LIBNAME): $(filter-out tests.rs,$(wildcard *.rs))
	rustc lib.rs

$(LIBNAME): config.rs

config.rs: gen-config
	./gen-config $(LUA_PCNAME) > $@

.INTERMEDIATE: gen-config
gen-config: config.c
	clang -o $@ $(CFLAGS) $<

test: $(wildcard *.rs) config.rs
	rustc --test lib.rs
	env RUST_THREADS=1 ./lua $(TESTNAME)

clean:
	rm -f lua $(LIBNAME) config.rs
	$(MAKE) -C examples clean

examples:
	$(MAKE) -C examples
