{
  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        libs = with pkgs; [
          bzip2
          webkitgtk
          gtk3
          cairo
          gdk-pixbuf
          libsoup
          glib
          openssl_3
        ];
        packages = with pkgs; [
          curl
          wget
          pkg-config
          dbus
          librsvg
        ] ++ libs;
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = packages;
          GIO_MODULE_DIR = "${pkgs.glib-networking}/lib/gio/modules/";
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libs;
        };
      });
}
