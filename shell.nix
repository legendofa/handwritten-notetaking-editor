{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
    buildInputs = [ gcc pkgconfig gtk3 ];
 	shellHook = XDG_DATA_DIRS=$GSETTINGS_SCHEMA_PATH;
}