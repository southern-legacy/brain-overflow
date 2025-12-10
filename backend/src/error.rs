pub mod api;
pub mod db;
pub mod fatal;

use std::error::Error;

use axum::response::{IntoResponse, Response};

/// ## 这个 trait 是一个 超类 trait，实现了它就相当于实现了自定义错误的的所有特性
///
/// ### 这个 trait 有关联类型
///
/// - `Kind`：此自定义错误具有哪些种类，这通常是一个枚举
///
/// ### 实现此 trait 之前，必须实现以下 trait：
///
/// - [`IntoResponse`]：对于 Path 错误，Json 错误等拦截器，这是必须实现的trait
/// - [`Error`]：这其实是一个附带的 trait，因为实现这个 trait 就必须实现 [`Debug`] 和 [`Display`](std::fmt::Display) 这两个 trait，而在日志中打印又需要这两个 trait
///
/// ### 同时必须为 [`Response`] 实现 [`From<Self>`] Self 的 trait
///
/// 这将使得这个 [`CustomError`] 能够在返回 [`Result<Response, Response>`] 的函数中使用 `?` 向上传递
///
/// ### 这个 trait 具有两个关联函数
///
/// - [`CustomError::kind`] 该函数能够获取错误种类
/// - [`CustomError::new`] 该函数能够通过自己对应的 Kind 构造一个自定义错误
///
pub trait CustomError
where
    Self: IntoResponse + Error + Sized,
    Response: From<Self>,
{
    type Kind;

    /// 获取自身的错误类型
    fn kind(&self) -> &Self::Kind;

    /// 创建一个新的自定义错误类型的实例
    fn new(kind: Self::Kind) -> Self;
}
