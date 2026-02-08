//! Telemetry provider initialization.

use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};
use opentelemetry_sdk::Resource;
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::config::TelemetryConfig;
use crate::error::TelemetryError;
use crate::guard::{self, TelemetryGuard};

fn map_init_err(e: impl std::fmt::Display) -> TelemetryError {
    TelemetryError::TracingInit(e.to_string())
}

/// Initialize telemetry with the given configuration.
pub fn init(config: TelemetryConfig) -> Result<TelemetryGuard, TelemetryError> {
    if guard::is_initialized() {
        return Err(TelemetryError::AlreadyInitialized);
    }

    if !config.enabled {
        guard::mark_initialized();
        return Ok(TelemetryGuard::new(None, None));
    }

    let resource = Resource::builder()
        .with_service_name(config.service_name.clone())
        .with_attribute(KeyValue::new(
            "service.version",
            config.service_version.clone(),
        ))
        .build();

    // Build trace provider
    let tracer_provider = if config.traces_enabled {
        let sampler = if (config.trace_sample_ratio - 1.0_f64).abs() < f64::EPSILON {
            Sampler::AlwaysOn
        } else if config.trace_sample_ratio <= 0.0 {
            Sampler::AlwaysOff
        } else {
            Sampler::TraceIdRatioBased(config.trace_sample_ratio)
        };

        let mut builder = SdkTracerProvider::builder()
            .with_sampler(sampler)
            .with_resource(resource.clone());

        #[cfg(feature = "otlp")]
        if config.otlp_endpoint.is_some() {
            use opentelemetry_otlp::SpanExporter;

            let exporter = SpanExporter::builder()
                .with_tonic()
                .build()
                .map_err(|e| TelemetryError::OtlpConfig(e.to_string()))?;
            builder = builder.with_batch_exporter(exporter);
        }

        let provider = builder.build();
        global::set_tracer_provider(provider.clone());
        Some(provider)
    } else {
        None
    };

    // Build meter provider
    let meter_provider = if config.metrics_enabled {
        let mut builder = SdkMeterProvider::builder().with_resource(resource);

        #[cfg(feature = "otlp")]
        if config.otlp_endpoint.is_some() {
            use opentelemetry_otlp::MetricExporter;
            use opentelemetry_sdk::metrics::PeriodicReader;

            let exporter = MetricExporter::builder()
                .with_tonic()
                .build()
                .map_err(|e| TelemetryError::OtlpConfig(e.to_string()))?;

            let reader = PeriodicReader::builder(exporter)
                .with_interval(std::time::Duration::from_secs(config.metrics_interval_secs))
                .build();
            builder = builder.with_reader(reader);
        }

        let provider = builder.build();
        global::set_meter_provider(provider.clone());
        Some(provider)
    } else {
        None
    };

    // Build tracing subscriber
    let env_filter = EnvFilter::try_new(&config.log_level).map_err(map_init_err)?;

    // Use a simpler init approach to avoid complex type nesting
    init_subscriber(env_filter, &tracer_provider, &meter_provider, &config)?;

    guard::mark_initialized();

    Ok(TelemetryGuard::new(meter_provider, tracer_provider))
}

fn init_subscriber(
    env_filter: EnvFilter,
    tracer_provider: &Option<SdkTracerProvider>,
    meter_provider: &Option<SdkMeterProvider>,
    config: &TelemetryConfig,
) -> Result<(), TelemetryError> {
    // Build layers based on configuration
    match (tracer_provider, meter_provider, config.log_to_stdout) {
        (Some(tp), Some(mp), true) => {
            let tracer = tp.tracer(config.service_name.clone());
            let otel_layer = OpenTelemetryLayer::new(tracer);
            let metrics_layer = MetricsLayer::new(mp.clone());

            if config.log_json {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(otel_layer)
                    .with(metrics_layer)
                    .with(tracing_subscriber::fmt::layer().json())
                    .try_init()
                    .map_err(map_init_err)
            } else {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(otel_layer)
                    .with(metrics_layer)
                    .with(tracing_subscriber::fmt::layer())
                    .try_init()
                    .map_err(map_init_err)
            }
        }
        (Some(tp), Some(mp), false) => {
            let tracer = tp.tracer(config.service_name.clone());
            let otel_layer = OpenTelemetryLayer::new(tracer);
            let metrics_layer = MetricsLayer::new(mp.clone());

            tracing_subscriber::registry()
                .with(env_filter)
                .with(otel_layer)
                .with(metrics_layer)
                .try_init()
                .map_err(map_init_err)
        }
        (Some(tp), None, true) => {
            let tracer = tp.tracer(config.service_name.clone());
            let otel_layer = OpenTelemetryLayer::new(tracer);

            if config.log_json {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(otel_layer)
                    .with(tracing_subscriber::fmt::layer().json())
                    .try_init()
                    .map_err(map_init_err)
            } else {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(otel_layer)
                    .with(tracing_subscriber::fmt::layer())
                    .try_init()
                    .map_err(map_init_err)
            }
        }
        (Some(tp), None, false) => {
            let tracer = tp.tracer(config.service_name.clone());
            let otel_layer = OpenTelemetryLayer::new(tracer);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(otel_layer)
                .try_init()
                .map_err(map_init_err)
        }
        (None, Some(mp), true) => {
            let metrics_layer = MetricsLayer::new(mp.clone());

            if config.log_json {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(metrics_layer)
                    .with(tracing_subscriber::fmt::layer().json())
                    .try_init()
                    .map_err(map_init_err)
            } else {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(metrics_layer)
                    .with(tracing_subscriber::fmt::layer())
                    .try_init()
                    .map_err(map_init_err)
            }
        }
        (None, Some(mp), false) => {
            let metrics_layer = MetricsLayer::new(mp.clone());

            tracing_subscriber::registry()
                .with(env_filter)
                .with(metrics_layer)
                .try_init()
                .map_err(map_init_err)
        }
        (None, None, true) => {
            if config.log_json {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(tracing_subscriber::fmt::layer().json())
                    .try_init()
                    .map_err(map_init_err)
            } else {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(tracing_subscriber::fmt::layer())
                    .try_init()
                    .map_err(map_init_err)
            }
        }
        (None, None, false) => tracing_subscriber::registry()
            .with(env_filter)
            .try_init()
            .map_err(map_init_err),
    }
}
