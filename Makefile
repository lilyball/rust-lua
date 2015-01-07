.PHONY: test all clean lib cargo-prep

LUA_PCNAME = $(if $(shell pkg-config --exists lua5.1 && echo yes),lua5.1,lua)
LUA_LIBNAME = $(firstword $(patsubst -llua%,lua%,$(filter -llua%,$(shell pkg-config --libs-only-l $(LUA_PCNAME)))))
LUA_LIBDIRS = $(patsubst -L%,%,$(shell pkg-config --libs-only-L $(LUA_PCNAME)))
CFLAGS += $(shell pkg-config --cflags $(LUA_PCNAME))

lib:
	cargo build

all: lib test doc

doc:
	cargo doc --no-deps

test:
	RUST_THREADS=1 cargo test -- $(TESTNAME)

clean:
	cargo clean

ifeq ($(OUT_DIR),)
cargo-prep:
	$(error cargo-prep must be called by cargo)
else
# the rest of the Makefile is only visible to Cargo

cargo-prep: $(OUT_DIR)/config.rs
	@echo "cargo:rustc-flags=-l $(LUA_LIBNAME) -L native=$(LUA_LIBDIRS)"

$(OUT_DIR)/config.rs: $(OUT_DIR)/gen-config
	echo "pub mod config {" > $@
	"$(OUT_DIR)"/gen-config >> $@
	echo "}" >> $@

$(OUT_DIR)/gen-config: src/config.c
	$(CC) -o $@ $(CFLAGS) $<

endif
