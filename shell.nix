{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell rec {
  buildInputs = with pkgs; [ libGL wayland libxkbcommon ];
  LD_LIBRARY_PATH = pkgs.lib.strings.join ":" (builtins.map (lib: "${lib}/lib") buildInputs);
}
