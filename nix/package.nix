{
  lib,
  stdenv,
  rustPlatform,
  pkg-config,
  makeWrapper,
  libdisplay-info_0_3,
  libinput,
  libgbm,
  libglvnd,
  libxkbcommon,
  libx11,
  libxi,
  libxcb,
  seatd,
  systemd,
  wayland,
  xwayland-satellite,
}:
let
  manifest = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = "driftline";
  version = manifest.package.version;
  src = lib.cleanSourceWith {
    src = ../.;
    filter = path: type: builtins.baseNameOf path != "target" && lib.cleanSourceFilter path type;
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = {
      "smithay-0.7.0" = "sha256-2aurK1dZAD49FTYURzZuQkBM07woZB/rNymF0fuSbR8=";
    };
  };

  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];
  buildInputs = [
    libdisplay-info_0_3
    libinput
    libgbm
    libxkbcommon
    seatd
    systemd
    wayland
  ];

  postPatch = ''
    substituteInPlace resources/driftline.service \
      --replace-fail 'ExecStart=driftline' "ExecStart=$out/bin/driftline"
    substituteInPlace resources/driftline.desktop \
      --replace-fail 'Exec=driftline-session' "Exec=$out/bin/driftline-session"
  '';

  preCheck = ''
    export LD_LIBRARY_PATH=${lib.makeLibraryPath [ wayland ]}
  '';

  installPhase = ''
    runHook preInstall
    make install PREFIX="$out" SYSCONFDIR="$out/share" TARGET_DIR=target/${stdenv.hostPlatform.rust.rustcTarget}
    wrapProgram "$out/bin/driftline" \
      --prefix LD_LIBRARY_PATH : ${
        lib.makeLibraryPath [
          libx11
          libxi
          libxcb
          libglvnd
          wayland
          libxkbcommon
          libgbm
        ]
      } \
      --suffix PATH : ${lib.makeBinPath [ xwayland-satellite ]}
    runHook postInstall
  '';

  doInstallCheck = true;
  installCheckPhase = ''
    runHook preInstallCheck
    "$out/bin/driftline" --version
    test -x "$out/bin/driftline-session"
    test -f "$out/share/driftline/config.reference.toml"
    grep -F "ExecStart=$out/bin/driftline --backend udev" "$out/lib/systemd/user/driftline.service"
    grep -F "Exec=$out/bin/driftline-session" "$out/share/wayland-sessions/driftline.desktop"
    runHook postInstallCheck
  '';

  passthru.providedSessions = [ "driftline" ];
  meta = {
    description = manifest.package.description;
    homepage = manifest.package.repository;
    license = lib.licenses.gpl3Plus;
    mainProgram = "driftline";
    platforms = lib.platforms.linux;
  };
}
