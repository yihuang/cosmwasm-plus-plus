{ sources ? import ./sources.nix, system ? builtins.currentSystem }:
import sources.nixpkgs {
  overlays = [
    (_: pkgs: rec {
      wasmvm = pkgs.rustPlatform.buildRustPackage rec {
        name = "wasmvm";
        src = sources.wasmvm;
        cargoSha256 = sha256:15z852h2fv73vjdnpis9yfxc3c3gqy6d67cp1m78xvhh7l5m2j0z;
        buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.libiconv
        ];
        doCheck = false;
      };
      wasmd = pkgs.buildGoModule rec {
        name = "wasmd";
        src = sources.wasmd;
        subPackages = [ "cmd/wasmd" ];
        vendorSha256 = sha256:1kqmm8f80l7lahlcq101z7ynj6r2fgp8s56k8dfq38dc0lp2pxkk;
        doCheck = false;
        postFixup = pkgs.lib.optionalString pkgs.stdenv.isLinux ''
          patchelf --set-rpath "${wasmvm}/lib" $out/bin/wasmd
        '' + pkgs.lib.optionalString pkgs.stdenv.isDarwin ''
          install_name_tool -change @rpath/libwasmvm.dylib "${wasmvm}/lib/libwasmvm.dylib" $out/bin/wasmd
        '';
      };
    })
    (_: pkgs: {
      pystarport = (import sources.chain-main { inherit system pkgs; }).pystarport-unbind;
    })
  ];
  config = { };
  inherit system;
}
