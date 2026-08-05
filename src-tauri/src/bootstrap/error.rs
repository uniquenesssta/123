use std::io;

pub(crate) const STARTUP_ERROR_MESSAGE: &str = "足球赛事模型平台启动失败";

pub(crate) fn io_error(error: impl ToString) -> io::Error {
    io::Error::other(error.to_string())
}

pub(crate) fn expect_startup(result: tauri::Result<()>) {
    result.expect(STARTUP_ERROR_MESSAGE);
}
