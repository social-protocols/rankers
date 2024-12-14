{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    # for `flake-utils.lib.eachDefaultSystem`
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        rDeps = with pkgs.rPackages; [
          rmarkdown
          DBI
          RSQLite
          lubridate
          stringr
          dplyr
          tidyr
          ggplot2
        ];
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ ];
          config = {
            allowUnfree = false;
            packageOverrides = super: let self = super.pkgs; in
            {
              rStudioWithPackages = super.rstudioWrapper.override{
                packages = rDeps;
              };
              # rWithPackages = super.rWrapper.override{
              #   packages = rDeps;
              # };
            };
          };
        };
        defaultBuildInputs = with pkgs; [
          entr
          just
          git

          rStudioWithPackages
          pandoc
          texlive.combined.scheme-full
        ];
      in
      {
        devShells.default = with pkgs; pkgs.mkShellNoCC {
          name = "dev-shell";
          buildInputs = defaultBuildInputs;
        };
      }
    );
}

