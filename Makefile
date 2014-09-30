include common.mk

.PHONY: test all clean examples lib

LUA_PCNAME = $(if $(shell pkg-config --exists lua5.1 && echo yes),lua5.1,lua)
LUA_LIBNAME = $(firstword $(patsubst -llua%,lua%,$(filter -llua%,$(shell pkg-config --libs-only-l $(LUA_PCNAME)))))
LUA_LIBDIRS = $(filter-out -L/usr/local/lib,$(shell pkg-config --libs-only-L $(LUA_PCNAME)))
CFLAGS += $(shell pkg-config --cflags $(LUA_PCNAME))

RUSTC := rustc
export RUSTFLAGS += -O $(LUA_LIBDIRS)

LIB_RS := $(filter-out tests.rs,$(wildcard *.rs))

lib: $(LIBNAME)

all: lib examples doc

$(LIBNAME): $(LIB_RS)
	$(RUSTC) $(RUSTFLAGS) src/lib.rs

$(LIBNAME): src/config.rs

src/config.rs: gen-config
	./gen-config $(LUA_LIBNAME) > $@

.INTERMEDIATE: gen-config
gen-config: src/config.c
	$(CC) -o $@ $(CFLAGS) $<

test: test-lua
	env RUST_THREADS=1 ./test-lua $(TESTNAME)

test-lua: $(wildcard src/*.rs) src/config.rs
	$(RUSTC) $(RUSTFLAGS) -o $@ --test src/lib.rs

clean:
	rm -f test-lua $(LIBNAME) src/config.rs
	rm -rf doc
	$(MAKE) -C examples clean

examples:
	$(MAKE) -C examples

examples/%:
	$(MAKE) -C examples $*

doc: $(LIB_RS) src/config.rs
	rustdoc src/lib.rs
	@touch doc
