{
  lib,
  buildGo125Module,
  fetchFromGitHub,
  ...
}:
let
  version = "0.0.0-dev";
in
buildGo125Module {
  pname = "signoz-backend";
  inherit version;

  src = fetchFromGitHub {
    owner = "SigNoz";
    repo = "signoz";
    rev = "8bfadbc1978c3acff9777c65f6152a0ec25087b9";
    hash = "sha256-oYJykkOuxBjb5jQmlUbibIz4DmoQDCmK02BNWQJBlDQ=";
  };

  proxyVendor = true;
  vendorHash = "sha256-NP1k+ED/hVZocGVwz1pLlAkQnTClkF7zIbJsunnY5/8=";

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
