include ../common.mk

.PHONY: all clean

EXAMPLES := simple sine simpleapi table

USES_COMMON := sine

all: $(EXAMPLES)

clean:
	rm -f $(EXAMPLES)

define BUILD_EXAMPLE
$(1): $(1).rs $(if $(filter $(1),$(USES_COMMON)),common.rs) ../$(LIBNAME)
	rustc -L .. $(1).rs
endef

$(foreach example,$(EXAMPLES),$(eval $(call BUILD_EXAMPLE,$(example))))

define BUILD_LIB
.PHONY: ../$(LIBNAME)
../$(LIBNAME):
	$(MAKE) -C .. $(LIBNAME)
endef

$(if $(shell $(MAKE) -C .. -q || echo no),$(eval $(call BUILD_LIB)))
