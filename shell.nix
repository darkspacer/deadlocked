# shell.nix — быстрая среда для проверок и тестов без flakes.
#
# Использование:
#   nix-shell                          # обычная dev-среда (сборка + запуск)
#   nix-shell --arg testOnly true      # среда, заточенная под `cargo test`
#   внутри: cargo check -p deadlocked
#           cargo test  -p deadlocked
#           cargo run   -p deadlocked --bin deadlocked
#
# Если у вас включены flakes, эквивалент: nix develop
# (см. devShells.default в flake.nix). Этот файл нужен именно для
# быстрых проверок на машинах/CI без flakes.

{
  pkgs ? import <nixpkgs> { },
  # Опционально: для той же цепочки инструментов, что и в flake.nix,
  # можно вызвать так:
  #   nix-shell -p ... --arg pkgs 'import <nixpkgs> { overlays = [ (import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz")) ]; }'
  # По умолчанию берём системный rust из nixpkgs — быстрее для проверок.
  testOnly ? false,
}:
let
  toolchain =
    if pkgs ? rust-bin then
      pkgs.rust-bin.stable.latest.minimal.override {
        extensions = [
          "rustfmt"
          "clippy"
        ];
      }
    else
      [
        pkgs.rustc
        pkgs.cargo
      ];

  # X11/Wayland/GL зависимости, нужные для компиляции и запуска
  buildDeps = with pkgs; [
    # cpal (via rodio) always links against alsa on Linux — without the
    # .dev pkg-config files `cargo build` dies in alsa-sys' build script
    alsa-lib.dev
    # NOTE: the `xorg` package set is deprecated in recent nixpkgs; use the
    # top-level renames (libx11, libxkbcommon, ...) to avoid eval warnings.
    libx11.dev
    # winit/x11-dl dlopen() these at runtime — without them the binary
    # panics on startup ("libxkbcommon-x11.so could not be loaded")
    libxkbcommon.dev
    # winit dlopen()s libX11-xcb.so.1 via x11-dl at event-loop creation —
    # missing => "Failed to load one of xlib's shared libraries".
    # x11-dl's build.rs resolves the pkg-config name `x11-xcb` and bakes its
    # `libdir` into the binary as a dlopen() fallback path, while at runtime
    # it tries plain soname lookups first. So we need BOTH the -dev package
    # (for pkg-config at build time) and the runtime .so directory on
    # LD_LIBRARY_PATH. In nixpkgs the actual libX11-xcb.so.1 lives in the
    # In current nixpkgs libX11-xcb.so.1 lives in the top-level `libx11`
    # package itself (the old `libX11_xcb` attribute no longer exists).
    libxext.dev
    libxrender.dev
    libXi.dev
    libxcursor.dev
    xorgproto
    wayland.dev
    libglvnd.dev
    vulkan-headers
    vulkan-loader.dev
    mesa
  ];

  runtimeLibs = pkgs.lib.makeLibraryPath (
    with pkgs;
    [
      alsa-lib
      libx11
      libxkbcommon
      libxext
      libxrender
      libXi
      libxcursor
      wayland
      libglvnd
      vulkan-loader
    ]
  );
in
pkgs.mkShell {
  packages =
    # toolchain — список [rustc cargo], если rust-overlay не подключён,
    # либо одна derivation от rust-bin — иначе
    (if builtins.isList toolchain then toolchain else [ toolchain ])
    ++ [
      pkgs.pkg-config
      pkgs.patchelf
      # cargo-husky (dev-зависимость репо) дергает git хуки при сборке —
      # без git в оболочке тесты падают
      pkgs.git
    ]
    ++ buildDeps
    ++ pkgs.lib.optionals (!testOnly) [
      # инструменты для ручной проверки in-game функционала
      pkgs.hyprland # hyprctl — правила оверлея Hyprland
      pkgs.gdb
    ]
    ++ pkgs.lib.optionals testOnly [
      pkgs.time
    ];

  shellHook = ''
    export LD_LIBRARY_PATH="${runtimeLibs}:$LD_LIBRARY_PATH"
    export PATH="$PWD/target/debug:$PATH"

    echo "deadlocked dev-shell ready"
    echo "  быстрая проверка:  cargo check -p deadlocked"
    echo "  тесты:             cargo test  -p deadlocked"
    echo "  запуск:            cargo run -p deadlocked --bin deadlocked"
  '';

  # Отключаем fortify-харденинг — patchelf/gdb-отладка бинарников работают
  # с ним нестабильно (совпадает с подходом в flake.nix).
  hardeningDisable = [ "fortify" ];
}
