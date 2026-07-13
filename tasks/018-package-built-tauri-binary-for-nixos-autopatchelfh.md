---
id: "018"
title: "Package built Tauri binary for NixOS (autoPatchelfHook or equivalent)"
status: in-progress
priority: medium
effort: medium
phase: ui-shell
dependencies: ["017"]
tags: ["tauri", "nix", "packaging"]
created_at: 2026-07-13
---

# Package built Tauri binary for NixOS (autoPatchelfHook or equivalent)

## Objective

Make the ui/ Tauri binary (built via the system rustup/cargo-tauri
toolchain, per the deliberate decision not to fight GTK/webkit into a
Nix devShell) actually runnable on the NixOS-based device. A binary built
with a normal distro toolchain doesn't run on NixOS as-is -- confirmed
concretely: `ldd` on the real debug binary showed it linked against
/lib/x86_64-linux-gnu/*.so and /lib64/ld-linux-x86-64.so.2, neither of
which exist on NixOS's non-FHS layout.

Deliberately NOT rebuilding the app via Nix's own toolchain -- that
reopens exactly the webkitgtk-in-Nix friction already decided against for
local dev. Instead: patch the externally-built binary's ELF metadata to
point at matching libraries from our own pinned nixpkgs, so the patched
result stays reproducible even though the build itself wasn't done
through Nix.

## Tasks

- [x] flake.nix packages.${system}.patch-ui-for-nixos -- a
      writeShellApplication wrapping patchelf, resolving the interpreter
      (pkgs.stdenv.cc.bintools.dynamicLinker) and RPATH
      (pkgs.lib.makeLibraryPath over the same GTK/webkit stack from the
      earlier devShell attempt, now genuinely appropriate as runtime deps
      here) from our pinned nixos-26.05, not an unpinned ad-hoc nixpkgs
      reference
- [x] Validated the tool itself builds (`nix build`), ShellCheck passed
      automatically as part of writeShellApplication
- [x] Ran it for real against a copy of the actual debug binary (never
      mutate the original in place without a backup -- patchelf modifies
      in place). Confirmed via patchelf --print-interpreter/--print-rpath
      after: interpreter is now a real Nix store glibc path, RPATH lists
      real Nix store paths for webkitgtk/gtk3/cairo/gdk-pixbuf/glib/dbus/
      openssl/etc. -- all paths that actually exist in this Nix store,
      not fabricated.

## Acceptance Criteria

- [x] Patched binary's ELF interpreter and RPATH point at valid Nix store
      paths, verified with patchelf, not assumed
- [ ] Patched binary actually launches on real NixOS -- cannot be
      validated from this sandbox (not NixOS), same honesty boundary as
      GPU detection and Ollama earlier. Needs testing on the actual
      device once hardware-configuration.nix (task 002) is real.
