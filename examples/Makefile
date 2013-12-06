.PHONY: all clean

all: simple

clean:
	rm simple

simple: simple.rs ../lua
	rustc -L .. simple.rs
