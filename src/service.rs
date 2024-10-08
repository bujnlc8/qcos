//! 查询bucket列表 方法见 [`crate::client::Client`#impl-Client-3]
use reqwest::header::{HeaderValue, HOST};

use crate::client::Client;
use crate::request::Request;
use crate::request::Response;

// 为了兼容以前的版本
pub struct Service;

impl Client {
    /**
    查询请求者名下的所有存储桶列表或特定地域下的存储桶列表
    见[文档](https://cloud.tencent.com/document/product/436/8291)
    # Examples
    ```
    use qcos::client::Client;
    async {
    let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
    let resp = client.get_bucket_list().await;
    assert!(resp.error_message.contains("403"));
    };
    ```
    */
    pub async fn get_bucket_list(&self) -> Response {
        let host = self.get_host_for_bucket_query();
        let mut headers = self.get_common_headers();
        headers.insert(HOST, HeaderValue::from_str(&host).unwrap());
        headers = self.get_headers_with_auth("get", "/", None, Some(headers), None);
        let resp = Request::get(
            format!("https://{}/", self.get_host_for_bucket_query()).as_str(),
            None,
            Some(&headers),
        )
        .await;
        self.make_response(resp)
    }
}
