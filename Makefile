PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
DATADIR = $(PREFIX)/share
LIBDIR = $(PREFIX)/lib
SYSCONFDIR ?= /etc

.PHONY: build install uninstall

TARGET_DIR = $(or $(CARGO_TARGET_DIR),target)

build:
	cargo build --release

install:
	install -Dm755 $(TARGET_DIR)/release/driftline $(DESTDIR)$(BINDIR)/driftline
	ln -sfn driftline $(DESTDIR)$(BINDIR)/driftwm
	install -Dm755 resources/driftline-session $(DESTDIR)$(BINDIR)/driftline-session
	ln -sfn driftline-session $(DESTDIR)$(BINDIR)/driftwm-session
	install -Dm644 resources/driftline.desktop $(DESTDIR)$(DATADIR)/wayland-sessions/driftline.desktop
	install -Dm644 resources/driftline-portals.conf $(DESTDIR)$(DATADIR)/xdg-desktop-portal/driftline-portals.conf
	install -Dm644 resources/driftline.service $(DESTDIR)$(LIBDIR)/systemd/user/driftline.service
	ln -sfn driftline.service $(DESTDIR)$(LIBDIR)/systemd/user/driftwm.service
	install -Dm644 resources/driftline-shutdown.target $(DESTDIR)$(LIBDIR)/systemd/user/driftline-shutdown.target
	ln -sfn driftline-shutdown.target $(DESTDIR)$(LIBDIR)/systemd/user/driftwm-shutdown.target
	install -Dm644 config.reference.toml $(DESTDIR)$(SYSCONFDIR)/driftline/config.reference.toml
	for f in extras/wallpapers/*.glsl extras/wallpapers/*/*.glsl; do \
		[ -e "$$f" ] || continue; \
		rel=$${f#extras/wallpapers/}; \
		install -Dm644 "$$f" "$(DESTDIR)$(DATADIR)/driftline/wallpapers/$$rel"; \
	done

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/driftline $(DESTDIR)$(BINDIR)/driftwm
	rm -f $(DESTDIR)$(BINDIR)/driftline-session $(DESTDIR)$(BINDIR)/driftwm-session
	rm -f $(DESTDIR)$(DATADIR)/wayland-sessions/driftline.desktop
	rm -f $(DESTDIR)$(DATADIR)/xdg-desktop-portal/driftline-portals.conf
	rm -f $(DESTDIR)$(LIBDIR)/systemd/user/driftline.service $(DESTDIR)$(LIBDIR)/systemd/user/driftwm.service
	rm -f $(DESTDIR)$(LIBDIR)/systemd/user/driftline-shutdown.target $(DESTDIR)$(LIBDIR)/systemd/user/driftwm-shutdown.target
	rm -rf $(DESTDIR)$(DATADIR)/driftline
	rm -rf $(DESTDIR)$(SYSCONFDIR)/driftline
