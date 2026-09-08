{
  description = "serenity-discord-bot development shell";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            packages =
              (with pkgs; [
                rustc
                cargo
                clippy
                rustfmt
                rust-analyzer

                sqlx-cli

                # Uncomment these two if you're using the `util-download` feature.
                # yt-dlp
                # ffmpeg
              ])

              # *Linux only*
              ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
                pkgs.mold # or lld, or gold, whatever linker you want
              ];

            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          };
        }
      );
    };
}
