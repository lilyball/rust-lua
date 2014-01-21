include common.mk

.PHONY: test all clean examples lib

LUA_PCNAME = $(if $(shell pkg-config --exists lua5.1 && echo yes),lua5.1,lua)
LUA_LIBNAME = $(firstword $(patsubst -llua%,lua%,$(filter -llua%,$(shell pkg-config --libs-only-l $(LUA_PCNAME)))))
CFLAGS += $(shell pkg-config --cflags $(LUA_PCNAME))

LIB_RS := $(filter-out tests.rs,$(wildcard *.rs))

lib: $(LIBNAME)

all: lib examples doc

$(LIBNAME): $(LIB_RS)
	rustc -O lib.rs

$(LIBNAME): config.rs

config.rs: gen-config
	./gen-config $(LUA_LIBNAME) > $@

.INTERMEDIATE: gen-config
gen-config: config.c
	clang -o $@ $(CFLAGS) $<

test: test-lua
	env RUST_THREADS=1 ./test-lua $(TESTNAME)

test-lua: $(wildcard *.rs) config.rs
	rustc -o $@ --test lib.rs

clean:
	rm -f test-lua $(LIBNAME) config.rs
	$(MAKE) -C examples clean

examples:
	$(MAKE) -C examples

doc: $(LIB_RS)
	rustdoc lib.rs
