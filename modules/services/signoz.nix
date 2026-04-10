{ inputs, ... }:
{
  perSystem =
    {
      self',
      pkgs,
      lib,
      ...
    }:
    let
      clickhouseUsers = pkgs.writeText "clickhouse-users.xml" ''
        <clickhouse replace="true">
            <profiles>
                <default>
                    <max_memory_usage>10000000000</max_memory_usage>
                    <use_uncompressed_cache>0</use_uncompressed_cache>
                    <load_balancing>in_order</load_balancing>
                    <log_queries>1</log_queries>
                </default>
            </profiles>
            <users>
                <default>
                    <profile>default</profile>
                    <networks><ip>::/0</ip></networks>
                    <quota>default</quota>
                    <access_management>1</access_management>
                    <named_collection_control>1</named_collection_control>
                    <show_named_collections>1</show_named_collections>
                    <show_named_collections_secrets>1</show_named_collections_secrets>
                </default>
            </users>
            <quotas>
                <default>
                    <interval>
                        <duration>3600</duration>
                        <queries>0</queries>
                        <errors>0</errors>
                        <result_rows>0</result_rows>
                        <read_rows>0</read_rows>
                        <execution_time>0</execution_time>
                    </interval>
                </default>
            </quotas>
        </clickhouse>
      '';

      clickhouseConfig = pkgs.writeText "clickhouse-config.xml" ''
        <clickhouse replace="true">
            <logger>
                <level>warning</level>
                <console>1</console>
            </logger>
            <display_name>cluster</display_name>
            <listen_host>127.0.0.1</listen_host>
            <http_port>8123</http_port>
            <tcp_port>9000</tcp_port>
            <user_directories>
                <users_xml><path>${clickhouseUsers}</path></users_xml>
            </user_directories>
            <distributed_ddl>
                <path>/clickhouse/task_queue/ddl</path>
            </distributed_ddl>
            <remote_servers>
                <cluster>
                    <shard>
                        <replica>
                            <host>localhost</host>
                            <port>9000</port>
                        </replica>
                    </shard>
                </cluster>
            </remote_servers>
            <!-- Embedded ClickHouse Keeper replaces external Zookeeper -->
            <keeper_server>
                <tcp_port>9181</tcp_port>
                <server_id>1</server_id>
                <log_storage_path>./data/clickhouse/coordination/log</log_storage_path>
                <snapshot_storage_path>./data/clickhouse/coordination/snapshots</snapshot_storage_path>
                <coordination_settings>
                    <operation_timeout_ms>10000</operation_timeout_ms>
                    <session_timeout_ms>30000</session_timeout_ms>
                </coordination_settings>
                <raft_configuration>
                    <server>
                        <id>1</id>
                        <hostname>localhost</hostname>
                        <port>9234</port>
                    </server>
                </raft_configuration>
            </keeper_server>
            <!-- Point zookeeper config to embedded Keeper -->
            <zookeeper>
                <node>
                    <host>localhost</host>
                    <port>9181</port>
                </node>
            </zookeeper>
            <macros>
                <shard>01</shard>
                <replica>01</replica>
            </macros>
            <path>./data/clickhouse/</path>
            <tmp_path>./data/clickhouse/tmp/</tmp_path>
            <user_files_path>./data/clickhouse/user_files/</user_files_path>
            <format_schema_path>./data/clickhouse/format_schemas/</format_schema_path>
        </clickhouse>
      '';

      collectorConfig = pkgs.writeText "otel-collector-config.yaml" ''
        receivers:
          otlp:
            protocols:
              grpc:
                endpoint: 0.0.0.0:4317
              http:
                endpoint: 0.0.0.0:4318
          prometheus:
            config:
              global:
                scrape_interval: 60s
              scrape_configs:
                - job_name: otel-collector
                  static_configs:
                    - targets:
                        - localhost:8888
                      labels:
                        job_name: otel-collector

        processors:
          batch:
            send_batch_size: 10000
            send_batch_max_size: 11000
            timeout: 10s
          resourcedetection:
            detectors: [env, system]
            timeout: 2s
          signozspanmetrics/delta:
            metrics_exporter: signozclickhousemetrics
            metrics_flush_interval: 60s
            latency_histogram_buckets:
              [
                100us,
                1ms,
                2ms,
                6ms,
                10ms,
                50ms,
                100ms,
                250ms,
                500ms,
                1000ms,
                1400ms,
                2000ms,
                5s,
                10s,
                20s,
                40s,
                60s,
              ]
            dimensions_cache_size: 100000
            aggregation_temporality: AGGREGATION_TEMPORALITY_DELTA
            enable_exp_histogram: true
            dimensions:
              - name: service.namespace
                default: default
              - name: deployment.environment
                default: default
              - name: signoz.collector.id
              - name: service.version
              - name: browser.platform
              - name: browser.mobile
              - name: k8s.cluster.name
              - name: k8s.node.name
              - name: k8s.namespace.name
              - name: host.name
              - name: host.type
              - name: container.name

        extensions:
          health_check:
            endpoint: 0.0.0.0:13133
          pprof:
            endpoint: 0.0.0.0:1777

        exporters:
          clickhousetraces:
            datasource: tcp://localhost:9000/signoz_traces
            low_cardinal_exception_grouping: false
            use_new_schema: true
          signozclickhousemetrics:
            dsn: tcp://localhost:9000/signoz_metrics
          clickhouselogsexporter:
            dsn: tcp://localhost:9000/signoz_logs
            timeout: 10s
            use_new_schema: true

        service:
          telemetry:
            logs:
              encoding: json
          extensions:
            - health_check
            - pprof
          pipelines:
            traces:
              receivers: [otlp]
              processors: [signozspanmetrics/delta, batch]
              exporters: [clickhousetraces]
            metrics:
              receivers: [otlp]
              processors: [batch]
              exporters: [signozclickhousemetrics]
            metrics/prometheus:
              receivers: [prometheus]
              processors: [batch]
              exporters: [signozclickhousemetrics]
            logs:
              receivers: [otlp]
              processors: [batch]
              exporters: [clickhouselogsexporter]
      '';
    in
    {
      process-compose.dev-platform.settings.processes = {
        clickhouse = {
          command = pkgs.writeShellApplication {
            name = "start-clickhouse";
            runtimeInputs = [ pkgs.clickhouse ];
            text = ''
              mkdir -p ./data/clickhouse
              clickhouse-server --config-file=${clickhouseConfig}
            '';
          };
          readiness_probe = {
            http_get = {
              host = "127.0.0.1";
              port = 8123;
              path = "/ping";
            };
            initial_delay_seconds = 3;
            period_seconds = 5;
            failure_threshold = 10;
          };
          availability = {
            restart = "on_failure";
            max_restarts = 5;
          };
        };

        signoz-telemetrystore-migrator = {
          command = pkgs.writeShellApplication {
            name = "run-signoz-migrations";
            runtimeInputs = [ self'.packages.signoz-otel-collector ];
            text = ''
              signoz-otel-collector migrate bootstrap
              signoz-otel-collector migrate sync up
              signoz-otel-collector migrate async up
            '';
          };
          environment = {
            SIGNOZ_OTEL_COLLECTOR_CLICKHOUSE_DSN = "tcp://127.0.0.1:9000";
            SIGNOZ_OTEL_COLLECTOR_CLICKHOUSE_CLUSTER = "cluster";
            SIGNOZ_OTEL_COLLECTOR_CLICKHOUSE_REPLICATION = "false";
          };
          depends_on.clickhouse.condition = "process_healthy";
          availability.restart = "no";
        };

        signoz-otel-collector = {
          command = pkgs.writeShellApplication {
            name = "start-signoz-otel-collector";
            runtimeInputs = [ self'.packages.signoz-otel-collector ];
            text = ''
              signoz-otel-collector migrate sync check
              signoz-otel-collector --config=${collectorConfig}
            '';
          };
          environment = {
            SIGNOZ_OTEL_COLLECTOR_CLICKHOUSE_DSN = "tcp://127.0.0.1:9000";
            SIGNOZ_OTEL_COLLECTOR_CLICKHOUSE_CLUSTER = "cluster";
            SIGNOZ_OTEL_COLLECTOR_CLICKHOUSE_REPLICATION = "false";
          };
          depends_on.clickhouse.condition = "process_healthy";
          readiness_probe = {
            http_get = {
              host = "127.0.0.1";
              port = 13133;
            };
            initial_delay_seconds = 5;
            period_seconds = 10;
            failure_threshold = 10;
          };
          availability = {
            restart = "on_failure";
            max_restarts = 5;
          };
        };

        signoz-backend = {
          command = pkgs.writeShellApplication {
            name = "start-signoz-backend";
            runtimeInputs = [ self'.packages.signoz-backend ];
            text = ''
              mkdir -p ./data/signoz
              signoz-backend server
            '';
          };
          environment = {
            SIGNOZ_TELEMETRYSTORE_CLICKHOUSE_DSN = "tcp://127.0.0.1:9000";
            SIGNOZ_TELEMETRYSTORE_CLICKHOUSE_CLUSTER = "cluster";
            SIGNOZ_SQLSTORE_SQLITE_PATH = "./data/signoz/signoz.db";
            SIGNOZ_TOKENIZER_JWT_SECRET = "secret";
            SIGNOZ_ALERTMANAGER_PROVIDER = "signoz";
            SIGNOZ_WEB_DIRECTORY = "${self'.packages.signoz-frontend}";
            SIGNOZ_WEB_ENABLED = "true";
          };
          depends_on.clickhouse.condition = "process_healthy";
          readiness_probe = {
            http_get = {
              host = "127.0.0.1";
              port = 8080;
              path = "/api/v1/health";
            };
            initial_delay_seconds = 10;
            period_seconds = 10;
            failure_threshold = 15;
          };
          availability = {
            restart = "on_failure";
            max_restarts = 5;
          };
        };
      };
    };
}
