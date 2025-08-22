use crate::app_config;

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    let logger_config = app_config::get_server().log();

    let logger = tracing_subscriber::registry()
        .with(EnvFilter::new(logger_config.level()))
        .with(
            pretty::PrettyLogger::new()
                .with_ansi(logger_config.with_ansi())
                .with_file(logger_config.with_file())
                .with_target(logger_config.with_target())
                .with_thread(logger_config.with_thread()),
        );

    if logger_config.dump_path().is_some() {
        let json = json::JsonLogger::new(logger_config.dump_path().unwrap());

        match json {
            Ok(json) => {
                logger
                    .with(
                        json.with_file(logger_config.with_file())
                            .with_target(logger_config.with_target())
                            .with_thread(logger_config.with_thread()),
                    )
                    .init();
            }
            Err(e) => {
                logger.init();
                tracing::error!("Cannot open the logger file! Details: {}", e);
            }
        }
    } else {
        logger.init();
    }
}

mod pretty {
    use crate::util::{
        AnsiColor::{self, *},
        AnsiString, AnsiStyle, FontStyle,
    };

    use std::u32;

    use chrono::Local;

    use tracing::{Level, span};
    use tracing_subscriber::Layer;

    #[derive(Default)]
    pub(super) struct PrettyLogger {
        with_target: bool,
        with_ansi: bool,
        with_file: bool,
        with_thread: bool,
    }

    impl<S> Layer<S> for PrettyLogger
    where
        S: tracing::Subscriber,
        S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
    {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            let style = self.severity_style(event);
            let prefix = style.decorate("|   ");
            let splitter = style.decorate("`-----------");
            let style = self.get_style(Some(Magenta), None, Some(FontStyle::new().bold(true)));
            self.print_level_label(event)
                .print_time(prefix, style)
                .print_target(event, prefix, style)
                .print_thread(event, prefix, style)
                .print_file(event, prefix, style)
                .print_spans(prefix, splitter, event, ctx);

            println!("{splitter}");
            event.record(&mut PrettyVisitor::new(self, event));
            println!("{splitter}\n");
        }

        fn on_new_span(
            &self,
            attrs: &span::Attributes<'_>,
            id: &span::Id,
            ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            let mut storage = PrettySpanFieldsStorage::new();
            attrs.record(&mut storage);
            if let Some(span) = ctx.span(id) {
                span.extensions_mut().insert(storage);
            }
        }
    }

    impl PrettyLogger {
        #[inline(always)]
        fn print_level_label(&self, event: &tracing::Event) -> &Self {
            let style = self.severity_label_style(event);
            let prefix = self.severity_style(event).decorate("*--");
            println!(
                "{prefix}{}{}{}",
                style.decorate("["),
                style.decorate(event.metadata().level().as_str()),
                style.decorate("]")
            );
            self
        }

        #[inline(always)]
        fn print_time(&self, prefix: AnsiString, style: AnsiStyle) -> &Self {
            println!(
                "{prefix}{:>8}: {}",
                style.decorate("time"),
                Local::now().to_rfc2822()
            );
            self
        }

        #[inline(always)]
        fn print_target(
            &self,
            event: &tracing::Event,
            prefix: AnsiString,
            style: AnsiStyle,
        ) -> &Self {
            if self.with_target {
                println!(
                    "{prefix}{:>8}: {}",
                    style.decorate("target"),
                    event.metadata().target()
                );
            }
            self
        }

        #[inline(always)]
        fn print_file(
            &self,
            event: &tracing::Event,
            prefix: AnsiString,
            style: AnsiStyle,
        ) -> &Self {
            if self.with_file {
                println!(
                    "{prefix}{:>8}: {}:{}",
                    style.decorate("file"),
                    event.metadata().file().unwrap_or("N/A"),
                    event.metadata().line().unwrap_or(u32::MAX)
                );
            }
            self
        }

        #[inline(always)]
        fn print_thread(
            &self,
            _event: &tracing::Event,
            prefix: AnsiString,
            style: AnsiStyle,
        ) -> &Self {
            if self.with_thread {
                println!(
                    "{prefix}{:>8}: {}@{:?}",
                    style.decorate("thread"),
                    std::thread::current().name().unwrap_or("N/A"),
                    std::thread::current().id(),
                );
            }
            self
        }

        fn print_spans<S>(
            &self,
            prefix: AnsiString,
            splitter: AnsiString,
            event: &tracing::Event<'_>,
            ctx: tracing_subscriber::layer::Context<'_, S>,
        ) -> &Self
        where
            S: tracing::Subscriber,
            S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
        {
            let inner_splitter = self
                .get_style(Some(Cyan), None, None)
                .decorate(splitter.get_content());
            let inner_prefix = self
                .get_style(Some(Cyan), None, None)
                .decorate(prefix.get_content());
            if let Some(scope) = ctx.event_scope(event) {
                println!("{splitter}");
                for span in scope.from_root() {
                    // span 的名字
                    println!(
                        "{prefix}{}",
                        self.get_style(Some(White), Some(Cyan), Some(FontStyle::new().bold(true)))
                            .decorate(if span.name().len() > 0 {
                                span.name()
                            } else {
                                "[N/A]"
                            })
                    );
                    println!(
                        "{prefix}{inner_prefix}{:>8}: {}",
                        self.get_style(Some(Cyan), None, None).decorate("target"),
                        span.metadata().target()
                    );
                    println!(
                        "{prefix}{inner_prefix}{:>8}: {}",
                        self.get_style(Some(Cyan), None, None).decorate("file"),
                        span.metadata().file().unwrap_or("N/A")
                    );
                    println!("{prefix}{inner_splitter}");
                    if let Some(storage) = span.extensions().get::<PrettySpanFieldsStorage>() {
                        for (k, v) in &storage.fields {
                            println!(
                                "{prefix}{inner_prefix}{:>8}: {v}",
                                self.get_style(Some(Cyan), None, None).decorate(k)
                            )
                        }
                    }
                    println!("{prefix}{inner_splitter}");
                }
            }

            self
        }

        #[inline(always)]
        fn severity_style<'a>(&self, event: &tracing::Event<'_>) -> AnsiStyle {
            match event.metadata().level() {
                &Level::TRACE => {
                    self.get_style(Some(Magenta), None, Some(FontStyle::new().bold(true)))
                }
                &Level::DEBUG => {
                    self.get_style(Some(Blue), None, Some(FontStyle::new().bold(true)))
                }
                &Level::INFO => self.get_style(Some(Green), None, None),
                &Level::WARN => self.get_style(Some(Yellow), None, None),
                &Level::ERROR => self.get_style(Some(Red), None, None),
            }
        }

        #[inline(always)]
        fn severity_label_style<'a>(&self, event: &tracing::Event<'_>) -> AnsiStyle {
            match event.metadata().level() {
                &Level::TRACE => self.get_style(
                    Some(BrightWhite),
                    Some(BrightMagenta),
                    Some(FontStyle::new().bold(true)),
                ),
                &Level::DEBUG => self.get_style(
                    Some(BrightWhite),
                    Some(BrightBlue),
                    Some(FontStyle::new().bold(true)),
                ),
                &Level::INFO => self.get_style(
                    Some(BrightBlack),
                    Some(BrightGreen),
                    Some(FontStyle::new().bold(true)),
                ),
                &Level::WARN => self.get_style(
                    Some(BrightBlack),
                    Some(BrightYellow),
                    Some(FontStyle::new().bold(true)),
                ),
                &Level::ERROR => self.get_style(
                    Some(BrightBlack),
                    Some(BrightRed),
                    Some(FontStyle::new().bold(true)),
                ),
            }
        }

        fn get_style(
            &self,
            fore: Option<AnsiColor>,
            back: Option<AnsiColor>,
            font: Option<FontStyle>,
        ) -> AnsiStyle {
            if !self.with_ansi {
                return AnsiStyle::new_vanilla();
            }

            let mut style = AnsiStyle::new();
            if fore.is_some() {
                style = style.with_fore(fore.unwrap());
            }
            if back.is_some() {
                style = style.with_back(back.unwrap());
            }
            if font.is_some() {
                style = style.merge_style(font.unwrap());
            }
            style
        }
    }

    impl PrettyLogger {
        pub(super) fn new() -> Self {
            Self::default()
        }

        pub(super) fn with_target(mut self, enabled: bool) -> Self {
            self.with_target = enabled;
            self
        }

        pub(super) fn with_ansi(mut self, enabled: bool) -> Self {
            self.with_ansi = enabled;
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

    struct PrettySpanFieldsStorage {
        fields: Vec<(&'static str, serde_json::Value)>,
    }

    impl PrettySpanFieldsStorage {
        fn new() -> Self {
            let mut fields = Vec::new();
            fields.reserve(4);
            Self { fields }
        }
    }

    impl tracing::field::Visit for PrettySpanFieldsStorage {
        fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
            self.fields.push((field.name(), serde_json::json!(value)));
        }

        fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
            self.fields.push((field.name(), serde_json::json!(value)));
        }

        fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
            self.fields.push((field.name(), serde_json::json!(value)));
        }

        fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
            self.fields.push((field.name(), serde_json::json!(value)));
        }

        fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
            self.fields.push((field.name(), serde_json::json!(value)));
        }

        fn record_error(
            &mut self,
            field: &tracing::field::Field,
            value: &(dyn std::error::Error + 'static),
        ) {
            self.fields
                .push((field.name(), serde_json::json!(value.to_string())));
        }

        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            self.fields
                .push((field.name(), serde_json::json!(format!("{:?}", value))));
        }
    }

    struct PrettyVisitor<'a> {
        config: &'a PrettyLogger,
        event: &'a tracing::Event<'a>,
    }

    impl<'a> PrettyVisitor<'a> {
        fn new(logger: &'a PrettyLogger, event: &'a tracing::Event<'_>) -> Self {
            Self {
                config: logger,
                event,
            }
        }
    }

    impl<'a> tracing::field::Visit for PrettyVisitor<'a> {
        fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
            let prefix = self.config.severity_style(self.event).decorate("|   ");
            println!(
                "{prefix}{:>8}: {}",
                self.config
                    .get_style(Some(Blue), None, Some(FontStyle::new().bold(true)))
                    .decorate(field.name()),
                value
            )
        }

        fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
            let prefix = self.config.severity_style(self.event).decorate("|   ");
            println!(
                "{prefix}{:>8}: {}",
                self.config
                    .get_style(Some(Blue), None, Some(FontStyle::new().bold(true)))
                    .decorate(field.name()),
                value
            )
        }

        fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
            let prefix = self.config.severity_style(self.event).decorate("|   ");
            println!(
                "{prefix}{:>8}: {}",
                self.config
                    .get_style(Some(Blue), None, Some(FontStyle::new().bold(true)))
                    .decorate(field.name()),
                value
            )
        }

        fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
            let prefix = self.config.severity_style(self.event).decorate("|   ");
            println!(
                "{prefix}{:>8}: {}",
                self.config
                    .get_style(Some(Blue), None, Some(FontStyle::new().bold(true)))
                    .decorate(field.name()),
                value
            )
        }

        fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
            let prefix = self.config.severity_style(self.event).decorate("|   ");
            println!(
                "{prefix}{:>8}: {}",
                self.config
                    .get_style(Some(Blue), None, Some(FontStyle::new().bold(true)))
                    .decorate(field.name()),
                value
            )
        }

        fn record_error(
            &mut self,
            field: &tracing::field::Field,
            value: &(dyn std::error::Error + 'static),
        ) {
            let prefix = self.config.severity_style(self.event).decorate("|   ");
            println!(
                "{prefix}{:>8}: {}",
                self.config
                    .get_style(Some(Blue), None, Some(FontStyle::new().bold(true)))
                    .decorate(field.name()),
                value
            )
        }

        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            let prefix = self.config.severity_style(self.event).decorate("|   ");
            println!(
                "{prefix}{:>8}: {:?}",
                self.config
                    .get_style(Some(Blue), None, Some(FontStyle::new().bold(true)))
                    .decorate(field.name()),
                value
            );
        }
    }
}

mod json {
    use std::{collections::BTreeMap, fs::File, io::Write, path::Path, sync::Arc};

    use chrono::Local;
    use serde_json::json;
    use std::fs;
    use tracing::span;
    use tracing_subscriber::Layer;

    pub(super) struct JsonLogger {
        with_target: bool,
        with_file: bool,
        with_thread: bool,
        file: Arc<File>,
    }

    impl JsonLogger {
        pub(super) fn new<P: AsRef<Path>>(dump_path: P) -> Result<Self, std::io::Error> {
            let log_path = dump_path.as_ref().to_path_buf();
            fs::create_dir_all(&log_path)?;

            let file = File::create(
                log_path.join(format!("{}.json", Local::now().format("%Y-%m-%d@%H-%M"))),
            )?;
            let file = Arc::new(file);
            Ok(Self {
                with_file: false,
                with_target: false,
                with_thread: false,
                file,
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
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
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
                .write(format!("{},\n", serde_json::to_string_pretty(&fields).unwrap()).as_bytes())
            {
                Ok(_) => return,
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
}
