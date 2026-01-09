{
  pkgs ? import <nixpkgs> { },
}:
(pkgs.buildFHSEnv {
  name = "verune";
  targetPkgs =
    pkgs: with pkgs; [
      gcc
      openssl
      rustc
      rustfmt
      clippy
      cargo
      rust-analyzer
      just
      just-lsp
      just-formatter
      pre-commit
      yaml-language-server
      git-cliff
      cargo-edit
      commitizen
    ];
  profile = ''
    export RUST_BACKTRACE=1
  '';
}).env
