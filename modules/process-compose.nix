{ inputs, ... }:
{
  imports = [
    inputs.process-compose-flake.flakeModule
  ];

  perSystem =
    { ... }:
    {
      process-compose.dev = {
        imports = [
          inputs.services-flake.processComposeModules.default
        ];

        settings.processes.ironstar-server = {
          command = "cargo run";
          readiness_probe = {
            http_get = {
              host = "127.0.0.1";
              port = 3000;
              path = "/health/ready";
            };
            initial_delay_seconds = 3;
            period_seconds = 5;
          };
          liveness_probe = {
            http_get = {
              host = "127.0.0.1";
              port = 3000;
              path = "/health/live";
            };
            period_seconds = 10;
          };
        };
      };
    };
}
