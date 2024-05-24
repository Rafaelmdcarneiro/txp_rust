{
  description = "txp";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }: let
    emptyOverlay = final: prev: {};
    txp-drv = pkgs:
      pkgs.rustPlatform.buildRustPackage {
        pname = "txp";
        version = "v0.1.0";

        src = ./.;

        buildFeatures = [ "ddsfile " "image" "dcv-color-primitives" ];

        cargoLock = {
          # Why I yes, I would like not writing the hash of my Cargo.lock very much.
          lockFile = ./Cargo.lock;
        };
      };
    txp-python-drv = pkgs: isNew: pythonPackages:
      pythonPackages.buildPythonPackage rec {
        pname = "txp";
        version = "v0.1.0";

        src = ./.;

        cargoDeps = pkgs.rustPlatform.importCargoLock {
          # Why I yes, I would like not writing the hash of my Cargo.lock very much.
          lockFile = ./Cargo.lock;
        };

        format = "pyproject";

        # HACK: maturinBuildHook is dumb and doesn't read pyproject.toml for some reason
        maturinBuildFlags = if isNew then ["--all-features"] else [''--cargo-extra-args="--all-features"''];
        nativeBuildInputs = with pkgs.rustPlatform; [cargoSetupHook maturinBuildHook];

        # needed for maturin
        propagatedBuildInputs = with pythonPackages; [cffi];
      };
    pythonOverride = prev: isNew: (prevArgs: {
      packageOverrides = let
        ourOverlay = new: old: {
          txp = txp-python-drv prev isNew old;
        };
      in
        prev.lib.composeExtensions
        prevArgs.packageOverrides or emptyOverlay
        ourOverlay;
    });
  in
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};
      in rec {
        packages = rec {
          txp = txp-drv pkgs;
          txp-python = txp-python-drv pkgs true pkgs.python310Packages;
          default = txp;
        };
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = ["rust-src" "cargo" "rustc"];
              targets = [ "x86_64-pc-windows-gnu" ];
            })
            gcc
          ];

          buildInputs = with pkgs; [
            maturin
            rust-analyzer
            cargo-semver-checks
            pkgsCross.mingwW64.stdenv.cc
            (pkgs.python3.withPackages (p:
              with p; [
                cffi
              ]))
          ];
          # RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
          #   pkgs.pkgsCross.mingwW64.windows.pthreads
          # ]);
        };
        devShells.python = pkgs.mkShell rec {
          buildInputs = with pkgs; [
            (pkgs.python310.withPackages (p:
              with p; [
                packages.txp-python
              ]))
          ];
        };
      }
    )
    // {
      overlays.default = final: prev: rec {
        txp = txp-drv prev;
        python3 = prev.python3.override (pythonOverride prev true);
        python310 = prev.python310.override (pythonOverride prev true);
        python39 = prev.python39.override (pythonOverride prev false);
      };
    };
}
