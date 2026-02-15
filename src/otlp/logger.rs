use crate::settings::{OtlpProtocol, Settings};
use opentelemetry_appender_tracing::layer;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

pub fn init_logger() -> SdkLoggerProvider {
    let config = &Settings::get();

    let exporter_builder = opentelemetry_otlp::LogExporter::builder();

    let otlp_exporter = match config.otlp_config.protocol {
        OtlpProtocol::Tonic => exporter_builder
            .with_tonic()
            .with_endpoint(config.otlp_config.collector_endpoint.clone())
            .build()
            .expect("OTLP Log Tonic build failed"),
        OtlpProtocol::Http => exporter_builder
            .with_http()
            .with_endpoint(config.otlp_config.collector_endpoint.clone())
            .build()
            .expect("OTLP Log HTTP build failed"),
    };

    let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name(config.service_name.clone())
                .build(),
        )
        .with_batch_exporter(otlp_exporter) // Batching is better for production OTLP
        .build();

    let filter_otel =
        EnvFilter::new(&config.otlp_config.log_level)
            .add_directive("reqwest=off".parse().unwrap());
    let otel_layer = layer::OpenTelemetryTracingBridge::new(&provider).with_filter(filter_otel);

    let filter_fmt = EnvFilter::new("debug").add_directive("opentelemetry=debug".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_filter(filter_fmt);

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    provider
}
