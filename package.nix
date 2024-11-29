# TODO: move this to nixpkgs
# This file aims to be a replacement for the nixpkgs derivation.

{ lib
, pkg-config
, rustPlatform
, fetchFromGitHub
, stdenv
, apple-sdk
, installShellFiles
, installShellCompletions ? stdenv.buildPlatform.canExecute stdenv.hostPlatform
, installManPages ? stdenv.buildPlatform.canExecute stdenv.hostPlatform
, gpgme
, buildNoDefaultFeatures ? false
, buildFeatures ? [ ]
}:

let
  version = "1.0.0";
  hash = "";
  cargoHash = "";
in

rustPlatform.buildRustPackage rec {
  inherit cargoHash version;
  inherit buildNoDefaultFeatures buildFeatures;

  pname = "mml";

  src = fetchFromGitHub {
    inherit hash;
    owner = "pimalaya";
    repo = "mml";
    rev = "v${version}";
  };

  nativeBuildInputs = [ pkg-config ]
    ++ lib.optional (installManPages || installShellCompletions) installShellFiles;

  buildInputs = [ ]
    ++ lib.optional stdenv.hostPlatform.isDarwin apple-sdk
    ++ lib.optional (builtins.elem "pgp-gpg" buildFeatures) gpgme;

  doCheck = false;
  auditable = false;

  # unit tests only
  cargoTestFlags = [ "--lib" ];

  postInstall = ''
    mkdir -p $out/share/{completions,man}
  '' + lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    "$out"/bin/mml man "$out"/share/man
  '' + lib.optionalString installManPages ''
    installManPage "$out"/share/man/*
  '' + lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    "$out"/bin/mml completion bash > "$out"/share/completions/mml.bash
    "$out"/bin/mml completion elvish > "$out"/share/completions/mml.elvish
    "$out"/bin/mml completion fish > "$out"/share/completions/mml.fish
    "$out"/bin/mml completion powershell > "$out"/share/completions/mml.powershell
    "$out"/bin/mml completion zsh > "$out"/share/completions/mml.zsh
  '' + lib.optionalString installShellCompletions ''
    installShellCompletion "$out"/share/completions/mml.{bash,fish,zsh}
  '';

  meta = rec {
    description = "CLI to convert MIME messages into/from Emacs MIME Meta Language";
    mainProgram = "mml";
    homepage = "https://github.com/pimalaya/mml";
    changelog = "${homepage}/blob/v${version}/CHANGELOG.md";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ soywod ];
  };
}
