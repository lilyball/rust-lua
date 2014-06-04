include ../common.mk

.PHONY: all clean

EXAMPLES := simple sine simpleapi table

USES_COMMON := sine

RUSTC := rustc
RUSTFLAGS += -L ..
unexport RUSTFLAGS

all: $(EXAMPLES)

clean:
	rm -f $(EXAMPLES) *.o

define BUILD_EXAMPLE
$(1): $(1).rs $(if $(filter $(1),$(USES_COMMON)),common.rs) ../$(LIBNAME)
ifeq (0,$(MAKELEVEL))
	$(MAKE) -C .. examples/$(1)
else
	$(RUSTC) $(RUSTFLAGS) $(1).rs
endif
endef

$(foreach example,$(EXAMPLES),$(eval $(call BUILD_EXAMPLE,$(example))))

define BUILD_LIB
.PHONY: ../$(LIBNAME)
../$(LIBNAME):
	$(MAKE) -C .. $(LIBNAME)
endef

$(if $(shell $(MAKE) -C .. -q || echo no),$(eval $(call BUILD_LIB)))
