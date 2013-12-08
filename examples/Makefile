.PHONY: all clean

EXAMPLES := simple sine simpleapi

USES_COMMON := sine

all: $(EXAMPLES)

clean:
	rm -f $(EXAMPLES)

define BUILD_EXAMPLE
$(1): $(1).rs $(if $(filter $(1),$(USES_COMMON)),common.rs)
	rustc -L .. $(1).rs
endef

$(foreach example,$(EXAMPLES),$(eval $(call BUILD_EXAMPLE,$(example))))
