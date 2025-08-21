use crate::app_config;

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    tracing_subscriber::registry()
        .with(EnvFilter::new(app_config::get_server().log_level()))
        .with(
            pretty::Logger::new()
                .with_ansi(true)
                .with_file(true)
                .with_level(true)
                .with_target(true)
                .with_mod_path(true)
                .with_thread(true),
        )
        .init();
}

mod pretty {
    use crate::util::{AnsiColor::*, AnsiString, AnsiStyle};

    use std::u32;

    use tracing::{Level, span};
    use tracing_subscriber::Layer;

    #[derive(Default)]
    pub(super) struct Logger {
        with_target: bool,
        with_level: bool,
        with_ansi: bool,
        with_file: bool,
        with_mod_path: bool,
        with_thread: bool,
    }

    impl<S> Layer<S> for Logger
    where
        S: tracing::Subscriber,
        S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
    {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            let (prefix, splitter);
            match self.with_ansi {
                true => {
                    let style = self.severity_style(event);
                    prefix = style.decorate("|   ");
                    splitter = style.decorate("`-----------");
                }
                false => {
                    prefix = AnsiString::new("|   ");
                    splitter = AnsiString::new("`-----------");
                }
            }

            self.print_level_label(event)
                .print_mod_path(event, prefix)
                .print_thread(event, prefix)
                .print_file(event, prefix)
                .print_spans(prefix, splitter, event, ctx);

            println!("{splitter}");
            event.record(&mut Visitor::new(self, event));
            println!("{splitter}\n");
        }

        fn on_new_span(
            &self,
            attrs: &span::Attributes<'_>,
            id: &span::Id,
            ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            let mut storage = SpanFieldsStorage::new();
            attrs.record(&mut storage);
            if let Some(span) = ctx.span(id) {
                span.extensions_mut().insert(storage);
            }
        }
    }

    impl Logger {
        #[inline(always)]
        #[allow(dead_code)]
        fn print_target(&self, event: &tracing::Event, prefix: AnsiString) -> &Self {
            if self.with_target {
                println!(
                    "{prefix}{:>8}: {}",
                    AnsiString::new("target").with_fore(Magenta),
                    event.metadata().target()
                );
            }
            self
        }

        #[inline(always)]
        fn print_level_label(&self, event: &tracing::Event) -> &Self {
            if self.with_level {
                let style = self.severity_style_label(event);
                let prefix = self.severity_style(event).decorate("*--");
                println!(
                    "{prefix}{}{}{}",
                    style.decorate("["),
                    style.decorate(event.metadata().level().as_str()),
                    style.decorate("]")
                );
            }
            self
        }

        #[inline(always)]
        fn print_file(&self, event: &tracing::Event, prefix: AnsiString) -> &Self {
            if self.with_file {
                println!(
                    "{prefix}{:>8}: {}:{}",
                    AnsiString::new("file").with_fore(Magenta),
                    event.metadata().file().unwrap_or("N/A"),
                    event.metadata().line().unwrap_or(u32::MAX)
                );
            }
            self
        }

        #[inline(always)]
        fn print_mod_path(&self, event: &tracing::Event, prefix: AnsiString) -> &Self {
            if self.with_mod_path {
                println!(
                    "{prefix}{:>8}: {}",
                    AnsiString::new("mod_path").with_fore(Magenta),
                    event.metadata().module_path().unwrap_or("N/A")
                );
            }
            self
        }

        #[inline(always)]
        fn print_thread(&self, _event: &tracing::Event, prefix: AnsiString) -> &Self {
            if self.with_thread {
                println!(
                    "{prefix}{:>8}: {}@{:?}",
                    AnsiString::new("thread").with_fore(Magenta),
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
            let inner_splitter = splitter.reset().with_fore(Cyan);
            let inner_prefix = prefix.reset().with_fore(Cyan);
            if let Some(scope) = ctx.event_scope(event) {
                println!("{splitter}");
                for span in scope.from_root() {
                    println!(
                        "{prefix}{}",
                        AnsiString::new(if span.name().len() > 0 {
                            span.name()
                        } else {
                            "[N/A]"
                        })
                        .with_fore(White)
                        .with_back(Cyan)
                    );
                    println!(
                        "{prefix}{inner_prefix}{:>8}: {}",
                        AnsiString::new("target").with_fore(Cyan),
                        span.metadata().target()
                    );
                    println!(
                        "{prefix}{inner_prefix}{:>8}: {}",
                        AnsiString::new("file").with_fore(Cyan),
                        span.metadata().file().unwrap_or("N/A")
                    );
                    println!("{prefix}{inner_splitter}");
                    if let Some(storage) = span.extensions().get::<SpanFieldsStorage>() {
                        for (k, v) in &storage.fields {
                            println!(
                                "{prefix}{inner_prefix}{:>8}: {v}",
                                AnsiString::new(k).with_fore(Cyan)
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
            if !self.with_ansi {
                return AnsiStyle::new();
            }
            match event.metadata().level() {
                &Level::TRACE => AnsiStyle::new().with_fore(Magenta),
                &Level::DEBUG => AnsiStyle::new().with_fore(Blue),
                &Level::INFO => AnsiStyle::new().with_fore(Green),
                &Level::WARN => AnsiStyle::new().with_fore(Yellow),
                &Level::ERROR => AnsiStyle::new().with_fore(Red),
            }
        }

        #[inline(always)]
        fn severity_style_label<'a>(&self, event: &tracing::Event<'_>) -> AnsiStyle {
            if !self.with_ansi {
                return AnsiStyle::new();
            }
            match event.metadata().level() {
                &Level::TRACE => AnsiStyle::new().with_fore(White).with_back(Magenta),
                &Level::DEBUG => AnsiStyle::new().with_fore(White).with_back(Blue),
                &Level::INFO => AnsiStyle::new().with_fore(White).with_back(Green),
                &Level::WARN => AnsiStyle::new().with_fore(Black).with_back(Yellow),
                &Level::ERROR => AnsiStyle::new().with_fore(White).with_back(Red),
            }
        }
    }

    impl Logger {
        pub(super) fn new() -> Self {
            Self::default()
        }

        pub(super) fn with_target(mut self, enabled: bool) -> Self {
            self.with_target = enabled;
            self
        }

        pub(super) fn with_level(mut self, enabled: bool) -> Self {
            self.with_level = enabled;
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

        pub(super) fn with_mod_path(mut self, enabled: bool) -> Self {
            self.with_mod_path = enabled;
            self
        }

        pub(super) fn with_thread(mut self, enabled: bool) -> Self {
            self.with_thread = enabled;
            self
        }
    }

    struct SpanFieldsStorage {
        fields: Vec<(&'static str, serde_json::Value)>,
    }

    impl SpanFieldsStorage {
        fn new() -> Self {
            let mut fields = Vec::new();
            fields.reserve(4);
            Self { fields }
        }
    }

    impl tracing::field::Visit for SpanFieldsStorage {
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

    struct Visitor<'a> {
        config: &'a Logger,
        event: &'a tracing::Event<'a>,
    }

    impl<'a> Visitor<'a> {
        fn new(logger: &'a Logger, event: &'a tracing::Event<'_>) -> Self {
            Self {
                config: logger,
                event,
            }
        }
    }

    impl<'a> tracing::field::Visit for Visitor<'a> {
        fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
            let prefix = self.config.severity_style(self.event).decorate("|   ");
            println!(
                "{prefix}{:>8}: {:?}",
                AnsiString::new(field.name()).with_fore(Blue),
                value
            )
        }

        fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
            let prefix = self.config.severity_style(self.event).decorate("|   ");
            println!(
                "{prefix}{:>8}: {:?}",
                AnsiString::new(field.name()).with_fore(Blue),
                value
            )
        }

        fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
            let prefix = self.config.severity_style(self.event).decorate("|   ");
            println!(
                "{prefix}{:>8}: {:?}",
                AnsiString::new(field.name()).with_fore(Blue),
                value
            )
        }

        fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
            let prefix = self.config.severity_style(self.event).decorate("|   ");
            println!(
                "{prefix}{:>8}: {:?}",
                AnsiString::new(field.name()).with_fore(Blue),
                value
            )
        }

        fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
            let prefix = self.config.severity_style(self.event).decorate("|   ");
            println!(
                "{prefix}{:>8}: {:?}",
                AnsiString::new(field.name()).with_fore(Blue),
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
                "{prefix}{:>8}: {:?}",
                AnsiString::new(field.name()).with_fore(Blue),
                value
            )
        }

        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            let prefix = self.config.severity_style(self.event).decorate("|   ");
            println!(
                "{prefix}{:>8}: {:?}",
                AnsiString::new(field.name()).with_fore(Blue),
                value
            );
        }
    }
}
