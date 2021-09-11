EXTRA_DIST = Cargo.toml \
	src/date_selector.json  \
	src/json_task.rs  \
	src/task.rs  \
	src/rofi.rs  \
	src/main.rs


EXTRA_PROGRAMS = target/@CARGO_TARGET_DIR@/rofitodo

# we need this dummy target for make dist
target_@CARGO_TARGET_DIR@_rofitodo_SOURCES =

all-local:
	cargo build --release

clean-local:
	-cargo clean

install-exec-local:
	$(MKDIR_P) $(DESTDIR)$(bindir)
	sudo $(INSTALL) -c -m 755 target/@CARGO_TARGET_DIR@/rofitodo $(DESTDIR)$(bindir)

DISTCLEANFILES =                                \
	$(null)

check:
