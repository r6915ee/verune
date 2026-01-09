{
  pkgs ? import <nixpkgs> { },
  # These are included by default because they're very common dependencies across runtimes overall.
  extraPkgs ? with pkgs; [
    openssl
    libyaml
    zlib
    libffi
  ],
}:
let
  mainPkg = pkgs.rustPlatform.buildRustPackage {
    name = "verune";
    src = pkgs.lib.cleanSource ./.;
    cargoLock.lockFile = ./Cargo.lock;
  };
in
pkgs.buildFHSEnv rec {
  name = "verune-fhs";
  executableName = "verune";
  targetPkgs =
    pkgs:
    [
      mainPkg
    ]
    ++ extraPkgs;
  runScript = "${mainPkg}/bin/${executableName}";
}
