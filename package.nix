# TODO: move this to nixpkgs
# This file aims to be a replacement for the nixpkgs derivation.

{
  buildFeatures ? [ ],
  buildNoDefaultFeatures ? false,
  buildPackages,
  fetchFromGitHub,
  installManPages ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installShellCompletions ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installShellFiles,
  lib,
  rustPlatform,
  stdenv,
}:

let
  version = "1.0.0";
  hash = "";
  cargoHash = "";

  emulator = stdenv.hostPlatform.emulator buildPackages;
  exe = stdenv.hostPlatform.extensions.executable;

in
rustPlatform.buildRustPackage {
  inherit cargoHash version buildNoDefaultFeatures;

  pname = "mml";

  src = fetchFromGitHub {
    inherit hash;
    owner = "pimalaya";
    repo = "mml";
    rev = "v${version}";
  };

  nativeBuildInputs = lib.optional (installManPages || installShellCompletions) installShellFiles;

  buildFeatures = buildFeatures ++ [ "cli" ];

  cargoTestFlags = [ "--lib" ];

  postInstall =
    lib.optionalString (lib.hasInfix "wine" emulator) ''
      export WINEPREFIX="''${WINEPREFIX:-$(mktemp -d)}"
      mkdir -p $WINEPREFIX
    ''
    + ''
      mkdir -p $out/share/{completions,man}
      ${emulator} "$out"/bin/mml${exe} manuals "$out"/share/man
      ${emulator} "$out"/bin/mml${exe} completions -d "$out"/share/completions bash elvish fish powershell zsh
    ''
    + lib.optionalString installManPages ''
      installManPage "$out"/share/man/*
    ''
    + lib.optionalString installShellCompletions ''
      installShellCompletion --cmd mml \
        --bash "$out"/share/completions/mml.bash \
        --fish "$out"/share/completions/mml.fish \
        --zsh "$out"/share/completions/_mml
    '';

  meta = {
    description = "CLI and lib for the Emacs MIME message Meta Language (MML), written in Rust";
    mainProgram = "mml";
    homepage = "https://github.com/pimalaya/mml";
    changelog = "https://github.com/pimalaya/mml/blob/master/CHANGELOG.md";
    license = [
      lib.licenses.mit
      lib.licenses.asl20
    ];
    maintainers = with lib.maintainers; [ soywod ];
  };
}
