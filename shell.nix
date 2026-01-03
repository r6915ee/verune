{
  pkgs ? import <nixpkgs> { },
}:
(pkgs.buildFHSEnv {
  name = "verune";
  targetPkgs =
    pkgs: with pkgs; [
      gcc
      rustc
      rustfmt
      clippy
      cargo
      rust-analyzer
      just
      just-lsp
      just-formatter
      pre-commit
    ];
}).env
