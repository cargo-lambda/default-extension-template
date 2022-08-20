{%- if logs -%}
use lambda_extension::{service_fn, Error, Extension, LambdaLog, LambdaLogRecord, SharedService};
use tracing::info;

async fn logs_extension(logs: Vec<LambdaLog>) -> Result<(), Error> {
    for log in logs {
        match log.record {
            LambdaLogRecord::Function(record) => info!("[logs] [function] {}", record),
            LambdaLogRecord::Extension(record) => info!("[logs] [extension] {}", record),
            _ => (),
        }
    }

    Ok(())
}
{%- else -%}
use lambda_extension::{service_fn, Error, LambdaEvent, NextEvent};

async fn my_extension(event: LambdaEvent) -> Result<(), Error> {
    match event.next {
        NextEvent::Shutdown(_e) => {
            // do something with the shutdown event
        }
        NextEvent::Invoke(_e) => {
            // do something with the invoke event
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
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    {% if logs -%}
    let logs_processor = SharedService::new(service_fn(logs_extension));
    Extension::new().with_logs_processor(logs_processor).run().await
    {%- else -%}
    let func = service_fn(my_extension);
    lambda_extension::run(func).await
    {%- endif %}
}