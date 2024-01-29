use lambda_extension::*;
use tracing::info;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
{%- assign use_events = false -%}
{%- if logs or telemetry -%}
    {%- if events -%}
        {%- assign use_events = true -%}
    {%- endif -%}
{%- else -%}
{%- assign use_events = true -%}
{%- endif -%}

{%- if logs %}

async fn logs_extension(logs: Vec<LambdaLog>) -> Result<(), Error> {
    for log in logs {
        match log.record {
            LambdaLogRecord::Function(record) => {
                info!(log_type = "function", record = ?record, "received function logs");
            }
            LambdaLogRecord::Extension(record) => {
                info!(log_type = "extension", record = ?record, "received extension logs");
            },
            _ignore_other => {},
        }
    }

    Ok(())
}
{%- endif -%}
{%- if telemetry %}

async fn telemetry_extension(events: Vec<LambdaTelemetry>) -> Result<(), Error> {
    for event in events {
        match event.record {
            LambdaTelemetryRecord::Function(record) => {
                info!(telemetry_type = "function", record = ?record, "received function telemetry");
            }
            _ignore_other => {},
        }
    }

    Ok(())
}
{%- endif -%}
{%- if use_events %}

async fn events_extension(event: LambdaEvent) -> Result<(), Error> {
    match event.next {
        NextEvent::Shutdown(e) => {
            info!(event_type = "shutdown", event = ?e, "shutting down");
        }
        NextEvent::Invoke(e) => {
            info!(event_type = "invoke", event = ?e, "invoking function");
        }
    }
    Ok(())
}
{%- endif %}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // The runtime logging can be enabled here by initializing `tracing` with `tracing-subscriber`
    // While `tracing` is used internally, `log` can be used as well if preferred.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    {% if logs -%}
    let logs_processor = SharedService::new(service_fn(logs_extension));
    {% endif -%}
    {% if telemetry -%}
    let telemetry_processor = SharedService::new(service_fn(telemetry_extension));
    {% endif %}
    Extension::new()
        {%- if use_events %}
        .with_events_processor(service_fn(events_extension))
        {%- endif -%}
        {%- if logs %}
        .with_logs_processor(logs_processor)
        {%- endif -%}
        {%- if telemetry %}
        .with_telemetry_processor(telemetry_processor)
        {%- endif %}
        .run()
        .await
}