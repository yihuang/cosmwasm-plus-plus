{ sources ? import ./sources.nix, system ? builtins.currentSystem }:
import sources.nixpkgs {
  overlays = [
    (_: pkgs: rec {
      wasmvm = pkgs.rustPlatform.buildRustPackage rec {
        name = "wasmvm";
        src = sources.wasmvm;
        cargoSha256 = sha256:1wpkxqck4kzyy67vp1b9kl9cdjqi7i9jx5m5x12r2dcsfp1bpzp8;
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
        vendorSha256 = sha256:1jrgbs28f1glnks3vak14w3ldnhp9iiil6pcqgjf2xar3627ln75;
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
