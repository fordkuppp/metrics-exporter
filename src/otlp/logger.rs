// use opentelemetry_otlp::WithExportConfig;
// use opentelemetry_sdk::logs::SdkLoggerProvider;
// use tracing_subscriber::prelude::*;
// use crate::settings::{Settings, OtlpProtocol};
//
// pub fn init_logger() -> SdkLoggerProvider {
//     let config = &Settings::get().otlp_config;
//
//     // 1. Setup Stdout Exporter (similar to your metrics style)
//     let stdout_exporter = opentelemetry_stdout::LogExporter::default();
//     let mut builder = SdkLoggerProvider::builder()
//         .with_simple_exporter(stdout_exporter);
//
//     // 2. Conditionally add OTLP Exporter
//     if config.enabled {
//         let exporter_builder = opentelemetry_otlp::LogExporter::builder();
//
//         let otlp_exporter = match config.protocol {
//             OtlpProtocol::Tonic => exporter_builder
//                 .with_tonic()
//                 .with_endpoint(config.collector_endpoint.clone())
//                 .build()
//                 .expect("OTLP Logs Tonic build failed"),
//             OtlpProtocol::Http => exporter_builder
//                 .with_http()
//                 .with_endpoint(config.collector_endpoint.clone())
//                 .build()
//                 .expect("OTLP Logs HTTP build failed"),
//         };
//
//         // Use batch exporter for OTLP to improve performance
//         builder = builder.with_batch_exporter(otlp_exporter);
//     }
//
//     let provider = builder.build();
//
//     // 3. Bridge to Tracing
//     // This allows tracing::info!, tracing::error!, etc. to flow into OTel
//     let otel_layer = Layer::new(&provider);
//
//     tracing_subscriber::registry()
//         .with(tracing_subscriber::filter::LevelFilter::INFO) // Ensure INFO is captured
//         .with(otel_layer)
//         .init();
//
//     provider
// }