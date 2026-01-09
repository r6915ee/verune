{
  pkgs ? import <nixpkgs> { },
  # It's so common as a dependency that OpenSSL and libffi are included by default in extraPkgs.
  extraPkgs ? with pkgs; [
    openssl
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
