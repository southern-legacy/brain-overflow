use std::{collections::BTreeMap, fs::File, io::Write, path::Path, sync::Arc};

use chrono::Local;
use serde_json::json;
use std::fs;
use tracing::span;
use tracing_subscriber::Layer;

use crate::logger::LogLevel;

pub(super) struct JsonLogger {
    with_target: bool,
    with_file: bool,
    with_thread: bool,
    file: Arc<File>,
    min_level: LogLevel,
}

#[derive(Default)]
struct JsonSpanFieldStorage {
    fields: BTreeMap<&'static str, serde_json::Value>,
}

struct JsonVisitor<'a> {
    fields: &'a mut BTreeMap<&'static str, serde_json::Value>,
}

impl<S> Layer<S> for JsonLogger
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if LogLevel::from(*event.metadata().level()) < self.min_level {
            return;
        }

        let mut fields = BTreeMap::new();
        let meta = event.metadata();
        fields.insert("level", json!(meta.level().as_str()));
        fields.insert("time", json!(Local::now().to_rfc2822()));
        fields.insert("target", json!(meta.target()));
        let curr_thread = std::thread::current();
        fields.insert(
            "thread",
            json!(format!(
                "{}@{:?}",
                curr_thread.name().unwrap_or("N/A"),
                curr_thread.id()
            )),
        );
        fields.insert(
            "file",
            json!(format!(
                "{}:{}",
                meta.file().unwrap_or("N/A"),
                meta.line().unwrap_or(u32::MAX)
            )),
        );
        let mut span_info = vec![];
        if let Some(scope) = ctx.event_scope(event) {
            for span in scope.from_root() {
                let span_meta = span.metadata();
                span_info.push(json!({
                        "target": span_meta.target(),
                        "file": format!("{}:{}", span_meta.file().unwrap_or("N/A"), span_meta.line().unwrap_or(u32::MAX)),
                        "fields": json!(
                            span.extensions()
                                .get::<JsonSpanFieldStorage>()
                                .unwrap_or(&JsonSpanFieldStorage::default())
                                .fields
                        )
                    }));
            }
        }
        let mut visitor = JsonVisitor::new(&mut fields);
        event.record(&mut visitor);

        fields.insert("spans", json!(span_info));

        match self
            .file
            .clone()
            .write_all(format!("{},\n", serde_json::to_string_pretty(&fields).unwrap()).as_bytes())
        {
            Ok(_) => (),
            Err(e) => println!("Cannot write to dump file, details: {e}"),
        }
    }

    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut storage = JsonSpanFieldStorage::new();
        attrs.record(&mut storage);
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(storage);
        }
    }
}

impl JsonLogger {
    pub(super) fn new<P: AsRef<Path>>(dump_path: P, min_level: LogLevel) -> Result<Self, std::io::Error> {
        let log_path = dump_path.as_ref().to_path_buf();
        fs::create_dir_all(&log_path)?;

        let file =
            File::create(log_path.join(format!("{}.json", Local::now().format("%Y.%m.%d@%H-%M"))))?;
        let file = Arc::new(file);
        Ok(Self {
            with_file: false,
            with_target: false,
            with_thread: false,
            file,
            min_level,
        })
    }

    pub(super) fn with_target(mut self, enabled: bool) -> Self {
        self.with_target = enabled;
        self
    }

    pub(super) fn with_file(mut self, enabled: bool) -> Self {
        self.with_file = enabled;
        self
    }

    pub(super) fn with_thread(mut self, enabled: bool) -> Self {
        self.with_thread = enabled;
        self
    }
}

impl JsonSpanFieldStorage {
    fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }
}

impl tracing::field::Visit for JsonSpanFieldStorage {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.fields.insert(field.name(), serde_json::json!(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields.insert(field.name(), serde_json::json!(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields.insert(field.name(), serde_json::json!(value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields.insert(field.name(), serde_json::json!(value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.fields.insert(field.name(), serde_json::json!(value));
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.fields
            .insert(field.name(), serde_json::json!(value.to_string()));
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields
            .insert(field.name(), serde_json::json!(format!("{:?}", value)));
    }
}

impl<'a> JsonVisitor<'a> {
    fn new(fields: &'a mut BTreeMap<&'static str, serde_json::Value>) -> Self {
        Self { fields }
    }
}

impl<'a> tracing::field::Visit for JsonVisitor<'a> {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.fields.insert(field.name(), serde_json::json!(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields.insert(field.name(), serde_json::json!(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields.insert(field.name(), serde_json::json!(value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields.insert(field.name(), serde_json::json!(value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.fields.insert(field.name(), serde_json::json!(value));
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.fields
            .insert(field.name(), serde_json::json!(value.to_string()));
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields
            .insert(field.name(), serde_json::json!(format!("{:?}", value)));
    }
}
