{
  lib,
  buildGo125Module,
  fetchFromGitHub,
  ...
}:
let
  version = "0.144.2";
in
buildGo125Module {
  pname = "signoz-otel-collector";
  inherit version;

  src = fetchFromGitHub {
    owner = "SigNoz";
    repo = "signoz-otel-collector";
    rev = "v${version}";
    hash = "sha256-Z32qaaYuNiuLs8DzJqvteAKSybv5G8tzSFV7HqtHoDg=";
  };

  vendorHash = "sha256-b5NAlYHKNM3UP5Z5VXGCN+HEJ59iDOPpMqWKvLUzwVQ=";

  subPackages = [ "cmd/signozotelcollector" ];

  ldflags = [
    "-s"
    "-w"
  ];

  # Rename the binary from 'signozotelcollector' to 'signoz-otel-collector'
  postInstall = ''
    mv $out/bin/signozotelcollector $out/bin/signoz-otel-collector
  '';

  meta = {
    description = "SigNoz distribution of OpenTelemetry Collector with custom processors and exporters";
    homepage = "https://github.com/SigNoz/signoz-otel-collector";
    license = lib.licenses.mit;
    mainProgram = "signoz-otel-collector";
  };
}
