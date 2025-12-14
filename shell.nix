{
  pkgs ? import <nixpkgs> { },
}:
(pkgs.buildFHSEnv {
  name = "verstring";
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
      asciidoctor-with-extensions
    ];
}).env
