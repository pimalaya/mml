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
  gpgme,
  libgpg-error,
  pkg-config,
  rustPlatform,
  stdenv,
}:

let
  gpg = builtins.elem "gpg" buildFeatures;

in
rustPlatform.buildRustPackage (finalAttrs: {
  __structuredAttrs = true;

  inherit buildNoDefaultFeatures;

  pname = "mml";
  version = "1.1.1";
  cargoHash = "";

  src = fetchFromGitHub {
    owner = "pimalaya";
    repo = finalAttrs.pname;
    tag = "v${finalAttrs.version}";
    hash = "";
  };

  # pkg-config hands the linker libgpgme but no rpath, leaving a binary that
  # cannot find it: not in postInstall, which runs it, nor once installed.
  env.NIX_LDFLAGS = lib.optionalString (gpg && !stdenv.hostPlatform.isWindows) (
    "-rpath " + lib.makeLibraryPath [ gpgme libgpg-error ]
  );

  # the build script links git2, whose libz-sys picks a system zlib as soon as
  # pkg-config finds one, then fails to load it for lack of an rpath: build
  # it statically, as it already is when pkg-config is absent.
  env.LIBZ_SYS_STATIC = "1";

  nativeBuildInputs = [ installShellFiles ] ++ lib.optional gpg pkg-config;

  buildInputs = lib.optionals gpg [
    gpgme
    libgpg-error
  ];

  # the binary lives behind the `cli` feature, off by default
  buildFeatures = buildFeatures ++ [ "cli" ];

  postInstall =
    let
      exe =
        if stdenv.buildPlatform.canExecute stdenv.hostPlatform then
          "$out/bin/${finalAttrs.meta.mainProgram}"
        else
          lib.getExe buildPackages.${finalAttrs.pname};
    in
    ''
      mkdir -p $out/share/{completions,man}
      ${exe} manual -d "$out"/share/man
      ${exe} completion -d "$out"/share/completions bash elvish fish powershell zsh
    ''
    + lib.optionalString installManPages ''
      installManPage "$out"/share/man/*
    ''
    + lib.optionalString installShellCompletions ''
      installShellCompletion --cmd ${finalAttrs.meta.mainProgram} \
        --bash "$out"/share/completions/${finalAttrs.meta.mainProgram}.bash \
        --fish "$out"/share/completions/${finalAttrs.meta.mainProgram}.fish \
        --zsh "$out"/share/completions/_${finalAttrs.meta.mainProgram}
    '';

  # disable the gpg integration tests: they need a gpg binary and a
  # writable home the sandbox does not provide
  cargoTestFlags = [ "--lib" ];

  meta = {
    description = "CLI and lib for the Emacs MIME message Meta Language (MML)";
    mainProgram = "mml";
    homepage = "https://github.com/pimalaya/${finalAttrs.pname}";
    changelog = "${finalAttrs.meta.homepage}/releases/tag/${finalAttrs.src.tag}";
    license = with lib.licenses; [
      asl20
      mit
    ];
    maintainers = with lib.maintainers; [ soywod ];
  };
})
