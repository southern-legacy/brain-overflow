use chrono::Local;

use crab_vault::color::{
    AnsiColor::{self, *},
    AnsiString, AnsiStyle, FontStyle,
};
use tracing::span;
use tracing_subscriber::Layer;

use crate::logger::LogLevel;

pub(super) struct PrettyLogger {
    with_target: bool,
    with_ansi: bool,
    with_file: bool,
    with_thread: bool,
    min_level: LogLevel,
}

struct PrettySpanFieldsStorage {
    fields: Vec<(&'static str, serde_json::Value)>,
}

struct PrettyVisitor<'a> {
    config: &'a PrettyLogger,
    event: &'a tracing::Event<'a>,
}

impl<S> Layer<S> for PrettyLogger
where
    S: tracing::Subscriber,
    S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if LogLevel::from(*event.metadata().level()) < self.min_level {
            return;
        }

        let style = self.severity_style(event);
        let prefix = style.decorate("|   ");
        let splitter = style.decorate("`-----------");
        let style = self.get_style(Some(Magenta), None, Some(FontStyle::new().bold(true)));
        self.print_level_label(event)
            .print_target(event, prefix, style)
            .print_thread(prefix, style)
            .print_file(event, prefix, style)
            .print_time(prefix, style)
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
    fn print_target(&self, event: &tracing::Event, prefix: AnsiString, style: AnsiStyle) -> &Self {
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
    fn print_file(&self, event: &tracing::Event, prefix: AnsiString, style: AnsiStyle) -> &Self {
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
    fn print_thread(&self, prefix: AnsiString, style: AnsiStyle) -> &Self {
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

    #[inline(always)]
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
                        .decorate(if !span.name().is_empty() {
                            span.name()
                        } else {
                            "[N/A]"
                        })
                );
                println!(
                    "{prefix}{inner_prefix}{:>8}: {}",
                    self.get_style(Some(Cyan), None, Some(FontStyle::new().bold(true)))
                        .decorate("target"),
                    span.metadata().target()
                );
                println!(
                    "{prefix}{inner_prefix}{:>8}: {}",
                    self.get_style(Some(Cyan), None, Some(FontStyle::new().bold(true)))
                        .decorate("file"),
                    span.metadata().file().unwrap_or("N/A")
                );
                println!("{prefix}{inner_splitter}");
                if let Some(storage) = span.extensions().get::<PrettySpanFieldsStorage>() {
                    for (k, v) in &storage.fields {
                        println!(
                            "{prefix}{inner_prefix}{:>8}: {v}",
                            self.get_style(Some(Cyan), None, Some(FontStyle::new().bold(true)))
                                .decorate(k)
                        )
                    }
                }
                println!("{prefix}{inner_splitter}");
            }
        }

        self
    }

    #[inline(always)]
    fn severity_style(&self, event: &tracing::Event<'_>) -> AnsiStyle {
        match *event.metadata().level() {
            tracing::Level::TRACE => {
                self.get_style(Some(Magenta), None, Some(FontStyle::new().bold(true)))
            }
            tracing::Level::DEBUG => {
                self.get_style(Some(Blue), None, Some(FontStyle::new().bold(true)))
            }
            tracing::Level::INFO => self.get_style(Some(Green), None, None),
            tracing::Level::WARN => self.get_style(Some(Yellow), None, None),
            tracing::Level::ERROR => self.get_style(Some(Red), None, None),
        }
    }

    #[inline(always)]
    fn severity_label_style(&self, event: &tracing::Event<'_>) -> AnsiStyle {
        match *event.metadata().level() {
            tracing::Level::TRACE => self.get_style(
                Some(BrightWhite),
                Some(BrightMagenta),
                Some(FontStyle::new().bold(true)),
            ),
            tracing::Level::DEBUG => self.get_style(
                Some(BrightWhite),
                Some(BrightBlue),
                Some(FontStyle::new().bold(true)),
            ),
            tracing::Level::INFO => self.get_style(
                Some(BrightBlack),
                Some(BrightGreen),
                Some(FontStyle::new().bold(true)),
            ),
            tracing::Level::WARN => self.get_style(
                Some(BrightBlack),
                Some(BrightYellow),
                Some(FontStyle::new().bold(true)),
            ),
            tracing::Level::ERROR => self.get_style(
                Some(BrightBlack),
                Some(BrightRed),
                Some(FontStyle::new().bold(true)),
            ),
        }
    }

    #[inline(always)]
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
        if let Some(fore) = fore {
            style = style.with_fore(fore);
        }
        if let Some(back) = back {
            style = style.with_back(back);
        }
        if let Some(font) = font {
            style = style.merge_style(font);
        }
        style
    }
}

impl PrettyLogger {
    pub(super) fn new(min_level: LogLevel) -> Self {
        Self {
            with_target: true,
            with_ansi: true,
            with_file: true,
            with_thread: true,
            min_level,
        }
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

impl PrettySpanFieldsStorage {
    fn new() -> Self {
        Self {
            fields: Vec::with_capacity(4),
        }
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
