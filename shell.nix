{
  nixpkgs ? <nixpkgs>,
  system ? builtins.currentSystem,
  pkgs ? import nixpkgs { inherit system; },
  pimalaya ? import (fetchTarball "https://github.com/pimalaya/nix/archive/master.tar.gz"),
  fenix ? import (fetchTarball "https://github.com/nix-community/fenix/archive/monthly.tar.gz") { },
}:

let
  # NOTE: gnupg and the gpgme headers are unconditional here, so the
  # `pgp-gpg` feature builds and tests in the dev shell without a
  # second entry point. The package keeps them behind the feature.
  inherit (pkgs)
    cargo-deny
    gnupg
    gpgme
    libgpg-error
    pkg-config
    zlib
    ;

  shell = pimalaya.mkShell {
    inherit
      nixpkgs
      system
      pkgs
      fenix
      ;
  };

in
shell.overrideAttrs (prev: {
  nativeBuildInputs = (prev.nativeBuildInputs or [ ]) ++ [
    pkg-config
  ];

  # NOTE: linking gpgme pulls zlib into the build script too, which
  # then needs it at run time. buildInputs only covers link time.
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    gpgme
    libgpg-error
    zlib
  ];

  buildInputs = (prev.buildInputs or [ ]) ++ [
    cargo-deny
    gnupg
    gpgme
    libgpg-error
    zlib
  ];
})
