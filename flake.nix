{
  description = "Subsnoic TUI music player";

  # Nixpkgs / NixOS version to use.
  inputs.nixpkgs.url = "nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let

      # to work with older version of flakes
      lastModifiedDate = self.lastModifiedDate or self.lastModified or "19700101";

      # Generate a user-friendly version number.
      version = builtins.substring 0 8 lastModifiedDate;

      # System types to support.
      supportedSystems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      # Helper function to generate an attrset '{ x86_64-linux = f "x86_64-linux"; ... }'.
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      # Nixpkgs instantiated for supported system types.
      nixpkgsFor = forAllSystems (system: import nixpkgs { inherit system; });

    in
    {

      # Provide some binary packages for selected system types.
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
        in
        {

          default = pkgs.rustPlatform.buildRustPackage {
            pname = "highpass";
            inherit version;
            # version = "";

            src = ./.;

            useFetchCargoVendor = true;
            cargoHash = "sha256-R+JdF5mLdcepdUCfVwNCrLoW2nJ0vk0XEQWMLjQGnbg=";

            libPath = pkgs.lib.makeLibraryPath [
              # pkgs.wayland
              # pkgs.libxkbcommon
              # pkgs.mpv
              pkgs.mpv-unwrapped
            ];

            buildInputs = [
              # pkgs.mpv
              pkgs.mpv-unwrapped
              # pkgs.fontconfig
              # pkgs.libxkbcommon
            ];
            nativeBuildInputs = [
              pkgs.makeWrapper
              # pkgs.mpv
              pkgs.mpv-unwrapped
              # pkgs.pkg-config
            ];

            postInstall = ''
              wrapProgram "$out/bin/highpass" --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath [
                pkgs.mpv-unwrapped
                pkgs.mpv-unwrapped.dev
              ]}"
            '';

            meta = with pkgs.lib; {
              description = "Subsnoic TUI music player";
              mainProgram = "highpass";
              homepage = "https://github.com/pinpox/highpass";
              license = licenses.gpl3Plus;
              maintainers = with maintainers; [ pinpox ];
              platforms = platforms.linux;
            };
          };

        }
      );

      # Add dependencies that are only needed for development
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              cargo
              rustc
              pkg-config
              mpv-unwrapped
              mpv-unwrapped.dev
            ];
            
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
            
            shellHook = ''
              export PKG_CONFIG_PATH="${pkgs.mpv-unwrapped.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
              export LD_LIBRARY_PATH="${pkgs.mpv-unwrapped}/lib:$LD_LIBRARY_PATH"
              export LIBRARY_PATH="${pkgs.mpv-unwrapped}/lib:$LIBRARY_PATH"
              export RUSTFLAGS="-L ${pkgs.mpv-unwrapped}/lib $RUSTFLAGS"
              
              # Additional MPV environment variables
              export MPV_HOME="${pkgs.mpv-unwrapped}"
              
              echo "MPV environment:"
              echo "  MPV version: $(${pkgs.mpv-unwrapped}/bin/mpv --version | head -1)"
              echo "  Library path: ${pkgs.mpv-unwrapped}/lib"
              echo "  Include path: ${pkgs.mpv-unwrapped.dev}/include"
            '';
          };
        }
      );
    };
}
