use crate::models::ResultWrap;

/// anyhow错误转为ResultWrap，方便前端拿到错误提示
pub fn wrap_anyhow<T, E: std::fmt::Display>(res: std::result::Result<T, E>) -> ResultWrap<T> {
    match res {
        Ok(v) => ResultWrap::success(v, "ok"),
        Err(e) => ResultWrap::fail(&format!("{}", e)),
    }
}
