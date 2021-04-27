{ sources ? import ./sources.nix, system ? builtins.currentSystem }:
import sources.nixpkgs {
  overlays = [
    (_: pkgs: {
      pystarport = (import sources.chain-main { inherit system; }).pystarport-unbind;
      wasmd = pkgs.buildGoModule rec {
        name = "wasmd";
        src = sources.wasmd;
        subPackages = [ "cmd/wasmd" ];
        vendorSha256 = sha256:0yacfm2sah3xmf8j3djryg1dgx2qbpqv4r469p56d2gabwdmws4l;
        doCheck = false;
      };
    })
  ];
  inherit system;
}
