use opentelemetry::global;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_otlp::WithExportConfig;
use crate::settings::{Settings, OtlpProtocol};

pub fn init_metrics() -> SdkMeterProvider {
    let config = &Settings::get().otlp_config;

    let stdout_exporter = opentelemetry_stdout::MetricExporter::default();
    let mut builder = SdkMeterProvider::builder()
        .with_periodic_exporter(stdout_exporter);

    if config.enabled {
        let exporter_builder = opentelemetry_otlp::MetricExporter::builder();

        let otlp_exporter = match config.protocol {
            OtlpProtocol::Tonic => exporter_builder
                .with_tonic()
                .with_endpoint(config.collector_endpoint.clone())
                .build()
                .expect("OTLP Tonic build failed"),
            OtlpProtocol::Http => exporter_builder
                .with_http()
                .with_endpoint(config.collector_endpoint.clone())
                .build()
                .expect("OTLP HTTP build failed"),
        };

        builder = builder.with_periodic_exporter(otlp_exporter);
    }

    let provider = builder.build();

    global::set_meter_provider(provider.clone());

    provider
}