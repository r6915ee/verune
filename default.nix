{
  pkgs ? import <nixpkgs> { },
  extraPkgs ? [ ],
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
