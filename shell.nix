let
  moz_overlay = import (builtins.fetchTarball
    "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz");
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  ruststable = (nixpkgs.latest.rustChannels.stable.rust.override {
    extensions = [
      "rust-src"
      "clippy-preview"
      "rls-preview"
      "rust-analysis"
      "rustfmt-preview"
    ];
  });
in with nixpkgs;
stdenv.mkDerivation {
  name = "rust";
  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
  buildInputs = [
    file
    llvmPackages.clang
    llvmPackages.libclang
    llvmPackages.llvm
    git-lfs
    chromaprint
    pkg-config
    ruststable
  ];
}
