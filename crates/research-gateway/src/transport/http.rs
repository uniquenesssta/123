use super::response::decode_response;
use super::{OpenAiTransport, TransportResponse};
use crate::{ApiKey, GatewayError, GatewayErrorCategory};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;
use std::sync::Once;
use std::time::Duration;

#[derive(Clone)]
pub struct ReqwestTransport {
    client: reqwest::Client,
}

impl ReqwestTransport {
    pub fn new() -> Result<Self, GatewayError> {
        install_rustls_crypto_provider();
        let client = reqwest::Client::builder()
            .user_agent(concat!(
                "football-match-model-platform/",
                env!("CARGO_PKG_VERSION")
            ))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| network_error(format!("无法初始化兼容 API HTTP客户端：{error}")))?;
        Ok(Self { client })
    }

    fn headers(api_key: &ApiKey) -> Result<HeaderMap, GatewayError> {
        let mut headers = HeaderMap::new();
        let authorization = HeaderValue::from_str(&format!("Bearer {}", api_key.expose()))
            .map_err(|_| {
                GatewayError::new(
                    GatewayErrorCategory::MissingCredential,
                    "兼容 API密钥包含无效字符",
                    false,
                    "重新保存Windows凭据管理器中的API密钥",
                )
            })?;
        headers.insert(AUTHORIZATION, authorization);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    async fn execute(
        request: reqwest::RequestBuilder,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        let response = request
            .timeout(timeout)
            .send()
            .await
            .map_err(map_reqwest_error)?;
        let status = response.status().as_u16();
        let provider_request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);
        let bytes = response.bytes().await.map_err(map_reqwest_error)?;
        decode_response(status, provider_request_id, &bytes)
    }
}

#[async_trait]
impl OpenAiTransport for ReqwestTransport {
    async fn post_json(
        &self,
        url: &str,
        api_key: &ApiKey,
        body: &Value,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        Self::execute(
            self.client
                .post(url)
                .headers(Self::headers(api_key)?)
                .json(body),
            timeout,
        )
        .await
    }

    async fn get_json(
        &self,
        url: &str,
        api_key: &ApiKey,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        Self::execute(
            self.client.get(url).headers(Self::headers(api_key)?),
            timeout,
        )
        .await
    }

    async fn post_empty(
        &self,
        url: &str,
        api_key: &ApiKey,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        Self::execute(
            self.client.post(url).headers(Self::headers(api_key)?),
            timeout,
        )
        .await
    }
}

static RUSTLS_PROVIDER_INSTALL: Once = Once::new();

fn install_rustls_crypto_provider() {
    RUSTLS_PROVIDER_INSTALL.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

fn map_reqwest_error(error: reqwest::Error) -> GatewayError {
    if error.is_timeout() {
        GatewayError::new(
            GatewayErrorCategory::Timeout,
            "OpenAI请求超时",
            true,
            "按退避策略重试；持续超时则缩小搜索范围或提高超时配置",
        )
    } else {
        network_error(format!("OpenAI网络请求失败：{error}"))
    }
}

fn network_error(message: impl Into<String>) -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::Network,
        message,
        true,
        "检查网络连接后按幂等键重试；应用历史记录和手工事实不受影响",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;
    use std::time::Instant;

    // One bounded loopback exchange in the existing unit-test target; no provider/API call.
    fn exchange(response: String, body_delay: Duration) -> (String, thread::JoinHandle<Vec<u8>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!(
            "http://{}/fixture?trace=original",
            listener.local_addr().unwrap()
        );
        listener.set_nonblocking(true).unwrap();
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            Instant::now() < deadline,
                            "loopback request was not received"
                        );
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("loopback accept failed: {error}"),
                }
            };
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut buffer = [0u8; 1024];
            loop {
                let count = stream.read(&mut buffer).unwrap();
                assert!(count > 0, "request closed before its body was complete");
                request.extend_from_slice(&buffer[..count]);
                if let Some(header_end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                    let headers =
                        String::from_utf8_lossy(&request[..header_end]).to_ascii_lowercase();
                    let length = headers
                        .lines()
                        .find_map(|line| {
                            line.strip_prefix("content-length:")
                                .map(|value| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    if request.len() >= header_end + 4 + length {
                        break;
                    }
                }
                assert!(request.len() < 16_384, "unexpected loopback request size");
            }
            let header_end = response.find("\r\n\r\n").unwrap() + 4;
            stream
                .write_all(&response.as_bytes()[..header_end])
                .unwrap();
            thread::sleep(body_delay);
            // Timeout test deliberately drops the reader before the body is sent.
            let _ = stream.write_all(&response.as_bytes()[header_end..]);
            request
        });
        (endpoint, worker)
    }

    #[test]
    fn authentication_headers_preserve_value_and_reject_invalid_characters() {
        let key = ApiKey::new("fixture-credential-value".into()).unwrap();
        let headers = ReqwestTransport::headers(&key).unwrap();
        assert_eq!(
            headers.get(AUTHORIZATION).unwrap(),
            "Bearer fixture-credential-value"
        );
        assert_eq!(headers.get(CONTENT_TYPE).unwrap(), "application/json");
        assert_eq!(headers.len(), 2);
        let key = ApiKey::new("fixture\0value".into()).unwrap();
        let error = ReqwestTransport::headers(&key).unwrap_err();
        assert_eq!(
            error,
            GatewayError::new(
                GatewayErrorCategory::MissingCredential,
                "兼容 API密钥包含无效字符",
                false,
                "重新保存Windows凭据管理器中的API密钥",
            )
        );
        assert!(!error.user_message.contains("fixture"));
    }

    #[tokio::test]
    async fn three_http_operations_preserve_method_url_headers_body_and_response() {
        let transport = ReqwestTransport::new().unwrap();
        let key = ApiKey::new("fixture-credential-value".into()).unwrap();
        let body = json!({"model": "fixture-model", "input": [{"text": "original"}]});
        for operation in 0..3 {
            let (url, worker) = exchange(
                "HTTP/1.1 429 Fixture\r\nContent-Length: 13\r\nx-request-id: req_fixture\r\nConnection: close\r\n\r\n{\"fixture\":1}".into(),
                Duration::ZERO,
            );
            let response = match operation {
                0 => {
                    transport
                        .post_json(&url, &key, &body, Duration::from_secs(5))
                        .await
                }
                1 => transport.get_json(&url, &key, Duration::from_secs(5)).await,
                _ => {
                    transport
                        .post_empty(&url, &key, Duration::from_secs(5))
                        .await
                }
            }
            .unwrap();
            let request = worker.join().unwrap();
            let header_end = request
                .windows(4)
                .position(|part| part == b"\r\n\r\n")
                .unwrap()
                + 4;
            let headers = String::from_utf8_lossy(&request[..header_end]).to_ascii_lowercase();
            let method = if operation == 1 { "get" } else { "post" };
            assert!(headers.starts_with(&format!("{method} /fixture?trace=original http/1.1\r\n")));
            assert!(headers.contains("authorization: bearer fixture-credential-value\r\n"));
            assert!(headers.contains("content-type: application/json\r\n"));
            assert!(headers.contains(&format!(
                "user-agent: football-match-model-platform/{}\r\n",
                env!("CARGO_PKG_VERSION")
            )));
            if operation == 0 {
                assert_eq!(
                    serde_json::from_slice::<Value>(&request[header_end..]).unwrap(),
                    body
                );
            } else {
                assert!(request[header_end..].is_empty());
            }
            assert_eq!(response.status, 429);
            assert_eq!(response.provider_request_id.as_deref(), Some("req_fixture"));
            assert_eq!(response.body, json!({"fixture": 1}));
        }
    }

    #[tokio::test]
    async fn redirects_are_returned_without_following_location() {
        let (url, worker) = exchange(
            "HTTP/1.1 307 Temporary Redirect\r\nLocation: /must-not-follow\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(),
            Duration::ZERO,
        );
        let transport = ReqwestTransport::new().unwrap();
        let key = ApiKey::new("fixture-credential-value".into()).unwrap();
        let response = transport
            .get_json(&url, &key, Duration::from_secs(5))
            .await
            .unwrap();
        worker.join().unwrap();
        assert_eq!(response.status, 307);
        assert_eq!(response.body, Value::Null);
    }

    #[tokio::test]
    async fn timeout_covers_response_body_and_keeps_original_recovery() {
        let (url, worker) = exchange(
            "HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nnull".into(),
            Duration::from_secs(2),
        );
        let transport = ReqwestTransport::new().unwrap();
        let key = ApiKey::new("fixture-credential-value".into()).unwrap();
        let error = transport
            .get_json(&url, &key, Duration::from_millis(500))
            .await
            .unwrap_err();
        let request = worker.join().unwrap();
        assert!(!request.is_empty());
        assert_eq!(
            error,
            GatewayError::new(
                GatewayErrorCategory::Timeout,
                "OpenAI请求超时",
                true,
                "按退避策略重试；持续超时则缩小搜索范围或提高超时配置",
            )
        );
    }

    #[tokio::test]
    async fn invalid_url_keeps_network_error_without_provider_metadata() {
        let transport = ReqwestTransport::new().unwrap();
        let key = ApiKey::new("fixture-credential-value".into()).unwrap();
        let error = transport
            .get_json("not a url", &key, Duration::from_secs(5))
            .await
            .unwrap_err();
        assert_eq!(error.category, GatewayErrorCategory::Network);
        assert!(error.user_message.starts_with("OpenAI网络请求失败："));
        assert!(error.recovery.retryable);
        assert_eq!(
            error.recovery.action,
            "检查网络连接后按幂等键重试；应用历史记录和手工事实不受影响"
        );
        assert_eq!(error.provider_status, None);
        assert_eq!(error.provider_code, None);
        assert!(!error.user_message.contains("fixture-credential-value"));
    }
}
