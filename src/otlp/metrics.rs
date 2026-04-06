use crate::settings::{OtlpProtocol, Settings};
use opentelemetry::global;
use opentelemetry_otlp::{WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::SdkMeterProvider;

pub fn init_metrics() -> SdkMeterProvider {
    let config = &Settings::get();
    let mut builder = SdkMeterProvider::builder().with_resource(
        Resource::builder()
            .with_service_name(config.service_name.clone())
            .build(),
    );

    if config.otlp_config.enable_stdout {
        let stdout_exporter = opentelemetry_stdout::MetricExporter::default();
        builder = builder.with_periodic_exporter(stdout_exporter);
    }
    let exporter_builder = opentelemetry_otlp::MetricExporter::builder();

    let otlp_exporter = match config.otlp_config.protocol {
        OtlpProtocol::Tonic => {
            let mut tonic_builder = exporter_builder
                .with_tonic()
                .with_endpoint(config.otlp_config.collector_endpoint.clone());
            if config.otlp_config.collector_endpoint.starts_with("https") {
                tonic_builder = tonic_builder
                    .with_tls_config(tonic::transport::ClientTlsConfig::new().with_enabled_roots());
            }
            tonic_builder.build().expect("OTLP Tonic build failed")
        }
        OtlpProtocol::Http => exporter_builder
            .with_http()
            .with_endpoint(format!("{}/v1/metrics", config.otlp_config.collector_endpoint))
            .build()
            .expect("OTLP HTTP build failed"),
    };

    builder = builder.with_periodic_exporter(otlp_exporter);

    let provider = builder.build();

    global::set_meter_provider(provider.clone());

    provider
}
