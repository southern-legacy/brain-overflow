use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename = "camelCase", default)]
pub struct PageOption {
    /// 第几页（zero-based），默认 0
    pub index: u64,

    /// 页面大小，默认 20
    pub page_size: u64,
}

impl Default for PageOption {
    fn default() -> Self {
        Self {
            index: 0,
            page_size: 20,
        }
    }
}

impl PageOption {
    #[inline]
    pub fn limit(self) -> i64 {
        self.page_size as i64
    }

    #[inline]
    pub fn offset(self) -> i64 {
        (self.index * self.page_size) as i64
    }
}
