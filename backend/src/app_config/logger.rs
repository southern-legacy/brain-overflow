use serde::{Deserialize, Serialize};

use crab_vault_logger::LogLevel;

use crate::{app_config::ConfigItem, error::fatal::FatalResult};

pub type LoggerConfig = StaticLoggerConfig;

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct StaticLoggerConfig {
    /// 最低的日志输出等级
    level: LogLevel,

    /// 彩色日志
    with_ansi: bool,

    /// 调用日志输出的文件
    with_file: bool,

    /// 调用日志输出的模块
    with_target: bool,

    /// 展示线程信息
    with_thread: bool,

    /// 日志文件输出到哪个文件夹下
    dump_path: Option<String>,

    /// 日志文件的最低输出等级
    dump_level: Option<LogLevel>,
}

impl ConfigItem for StaticLoggerConfig {
    type RuntimeConfig = LoggerConfig;

    fn into_runtime(self) -> FatalResult<Self::RuntimeConfig> {
        Ok(self)
    }
}

impl Default for StaticLoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Trace,
            dump_path: None,
            dump_level: None,
            with_ansi: true,
            with_file: true,
            with_target: true,
            with_thread: true,
        }
    }
}

impl StaticLoggerConfig {
    #[inline]
    pub fn level(&self) -> LogLevel {
        self.level
    }

    #[inline]
    pub fn dump_path(&self) -> Option<&str> {
        match &self.dump_path {
            Some(val) => Some(val),
            None => None,
        }
    }

    /// dump_level 完全依赖于 `dump_path` ，只有在设置 `dump_path` 之后，才会有 `dump_path` ，否则此值无意义
    ///
    /// ### 这也意味着如果 `dump_path.is_some()` 成立，这个函数的返回值就可以直接 `unwrap()`，如果未指定，将返回 Warn
    #[inline]
    pub fn dump_level(&self) -> Option<LogLevel> {
        if self.dump_path().is_some() {
            match self.dump_level {
                Some(val) => Some(val),
                None => Some(LogLevel::Warn),
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn with_ansi(&self) -> bool {
        self.with_ansi
    }

    #[inline]
    pub fn with_file(&self) -> bool {
        self.with_file
    }

    #[inline]
    pub fn with_target(&self) -> bool {
        self.with_target
    }

    #[inline]
    pub fn with_thread(&self) -> bool {
        self.with_thread
    }
}
