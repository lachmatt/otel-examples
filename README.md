# Splunk AppDynamics SaaS via the OpenTelemetry Collector

This repository contains a docker compose setup to send telemetry data to AppDynamics SaaS using the OpenTelemetry Collector.
Following environment variables must be set before starting the services:

- APPD_HOST
- APPD_PORT
- APPD_ACCOUNT
- APPD_API_KEY
- APPD_REGION
- APPD_SERVICE_NAMESPACE
