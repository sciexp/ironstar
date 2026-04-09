{
  inputs,
  lib,
  buildGo125Module,
  ...
}:
let
  version = "0.0.0-dev";
in
buildGo125Module {
  pname = "signoz-backend";
  inherit version;

  src = inputs.signoz-src;

  vendorHash = "sha256-nrTr8MmLA4E9/f+h7kvHxMmNC+aMmNL41ik53nq0pjU=";

  subPackages = [ "cmd/community" ];

  tags = [ "timetzdata" ];

  ldflags = [
    "-s"
    "-w"
    "-X github.com/SigNoz/signoz/pkg/version.version=${version}"
    "-X github.com/SigNoz/signoz/pkg/version.variant=community"
  ];

  # Rename the binary from 'community' to 'signoz-backend'
  postInstall = ''
    mv $out/bin/community $out/bin/signoz-backend
  '';

  meta = {
    description = "SigNoz backend server (community edition)";
    homepage = "https://github.com/SigNoz/signoz";
    license = lib.licenses.mit;
    mainProgram = "signoz-backend";
  };
}
