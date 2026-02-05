{ inputs, ... }:
{
  imports = [
    inputs.process-compose-flake.flakeModule
  ];

  perSystem =
    { pkgs, lib, ... }:
    {
      process-compose.dev = {
        imports = [
          inputs.services-flake.processComposeModules.default
        ];

        settings = {
          log_level = "info";
          log_length = 1000;

          environment = {
            RUST_BACKTRACE = "1";
            RUST_LOG = "debug,sqlx=warn,tower_http=debug";
          };

          processes = {
            # One-shot: ensure data directory exists for SQLite (rwc mode handles DB creation)
            db-init = {
              command = pkgs.writeShellApplication {
                name = "db-init";
                text = ''
                  mkdir -p data
                  echo "data directory ready"
                '';
              };
              availability.restart = "no";
            };

            # Frontend: Rolldown/PostCSS dev watcher for CSS/JS bundling
            frontend = {
              command = pkgs.writeShellApplication {
                name = "frontend-dev";
                runtimeInputs = [
                  pkgs.pnpm
                  pkgs.nodejs
                ];
                text = ''
                  cd web-components
                  pnpm install --frozen-lockfile
                  pnpm dev
                '';
              };
              availability = {
                restart = "on_failure";
                backoff_seconds = 2;
              };
            };

            # Backend: Rust server with cargo-watch for auto-rebuild
            backend = {
              command = pkgs.writeShellApplication {
                name = "backend-dev";
                runtimeInputs = [ pkgs.cargo-watch ];
                text = ''
                  cargo watch -x 'run --package ironstar'
                '';
              };
              depends_on = {
                db-init.condition = "process_completed_successfully";
                frontend.condition = "process_started";
              };
              readiness_probe = {
                http_get = {
                  host = "127.0.0.1";
                  port = 3000;
                  path = "/health/ready";
                };
                initial_delay_seconds = 5;
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
              environment = {
                IRONSTAR_DATABASE_URL = "sqlite:./data/ironstar.db?mode=rwc";
              };
            };

            # Hot reload: watch source and static assets, trigger browser refresh
            # via curl to the backend. The browser polls /reload with datastar retry;
            # cargo-watch restart causes a transient connection drop that triggers
            # the browser to reconnect, achieving live reload without a separate
            # websocket server.
            hotreload = {
              command = pkgs.writeShellApplication {
                name = "hotreload-dev";
                runtimeInputs = [
                  pkgs.cargo-watch
                  pkgs.curl
                ];
                text = ''
                  cargo watch \
                    -w crates \
                    -w static/dist \
                    -w web-components/styles \
                    -s 'curl -sf http://127.0.0.1:3000/health/ready > /dev/null && echo "reload triggered" || echo "server not ready"'
                '';
              };
              depends_on = {
                backend.condition = "process_healthy";
              };
              availability = {
                restart = "on_failure";
                backoff_seconds = 2;
              };
            };
          };
        };

        # Observability services via services-flake modules

        services.prometheus.prom1 = {
          enable = true;
          port = 9090;
          listenAddress = "127.0.0.1";
          extraConfig = {
            scrape_configs = [
              {
                job_name = "ironstar";
                scrape_interval = "10s";
                static_configs = [
                  {
                    targets = [ "127.0.0.1:3000" ];
                    labels = {
                      instance = "ironstar-dev";
                    };
                  }
                ];
                # /metrics will 404 until s5j.5 implements the endpoint;
                # Prometheus handles this gracefully with scrape errors.
                metrics_path = "/metrics";
              }
              {
                job_name = "prometheus";
                static_configs = [
                  {
                    targets = [ "127.0.0.1:9090" ];
                  }
                ];
              }
            ];
          };
        };

        services.grafana.graf1 = {
          enable = true;
          http_port = 3001;
          domain = "127.0.0.1";
          extraConf = {
            security = {
              admin_user = "admin";
              admin_password = "admin";
            };
            # Disable login form for frictionless dev access
            "auth.anonymous" = {
              enabled = "true";
              org_role = "Admin";
            };
          };
          datasources = [
            {
              name = "Prometheus";
              type = "prometheus";
              access = "proxy";
              url = "http://127.0.0.1:9090";
              isDefault = true;
              editable = true;
            }
          ];
        };
      };
    };
}
