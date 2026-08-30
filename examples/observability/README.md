# Relay observability example

This optional stack receives Relay traces, metrics, and logs through OTLP/HTTP. It is not required by Relay.

```bash
docker compose up -d
```

Configure Relay:

```env
PJ_OTEL_ENABLED=true
OTEL_EXPORTER_OTLP_ENDPOINT=http://host.docker.internal:4318
OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
OTEL_SERVICE_NAME=promptjang-relay
```

Open Jaeger at <http://localhost:16686> and Prometheus at <http://localhost:9090>. Logs are emitted by the Collector debug exporter. Replace the exporters in `otel-collector.yaml` with any OTLP-compatible backend.
