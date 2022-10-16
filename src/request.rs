//! 请求封装
use std::collections::HashMap;
use std::fmt::Display;

use bytes::Bytes;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Body;
use serde_json::value::Value;
use std::convert::From;
use std::str::FromStr;
use std::time::Duration;

use reqwest;

pub struct Request;
use serde;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct InitiateMultipartUploadResult {
    #[serde(rename(deserialize = "Bucket"))]
    bucket: String,
    #[serde(rename(deserialize = "Key"))]
    key: String,
    #[serde(rename(deserialize = "UploadId"))]
    pub upload_id: String,
}

/// ```
/// use qcos::request::{CompleteMultipartUpload, Part};
/// use quick_xml::se::to_string;
/// let objs = CompleteMultipartUpload{part:vec![Part{part_number: 1, etag: "abc".to_string()}, Part{part_number: 2, etag: "abc".to_string()}]};
/// let s = to_string(&objs).unwrap();
/// assert_eq!(s, r#"<CompleteMultipartUpload><Part><PartNumber>1</PartNumber><ETag>abc</ETag></Part><Part><PartNumber>2</PartNumber><ETag>abc</ETag></Part></CompleteMultipartUpload>"#)
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CompleteMultipartUpload {
    #[serde(rename(serialize = "Part"))]
    pub part: Vec<Part>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
pub struct Part {
    #[serde(rename = "$unflatten=PartNumber")]
    pub part_number: u64,
    #[serde(rename = "$unflatten=ETag")]
    pub etag: String,
}

/// 错误码
#[derive(Debug, PartialEq)]
pub enum ErrNo {
    /// 操作成功
    SUCCESS = 0,
    /// 其他错误
    OTHER = 10000,
    /// http status code 相关错误
    STATUS = 10001,
    /// 解码相关错误
    DECODE = 10002,
    /// 连接相关错误
    CONNECT = 10003,
    /// 编码相关错误
    ENCODE = 20001,
    /// IO错误
    IO = 20002,
}

/// 请求方法
#[derive(Debug, Eq, PartialEq)]
pub enum Method {
    Get,
    Post,
    Delete,
    Put,
    Head,
}

/// # Examples
/// ```
/// use qcos::request::ErrNo;
/// println!("{:#?}", ErrNo::OTHER);
/// ```
impl ToString for ErrNo {
    fn to_string(&self) -> String {
        format!("{:#?}", self)
    }
}

/// http请求返回类型，无论成功还是失败都返回该类型，根据`error_no`可区分是否成功
#[derive(Debug)]
pub struct Response {
    /// 错误码
    pub error_no: ErrNo,
    /// 错误信息
    pub error_message: String,
    /// 接口返回信息，当接口返回错误时也可能有值
    pub result: Bytes,
    /// 接口返回的headers, 有些接口需要拿到头部信息进行校验
    pub headers: HashMap<String, String>,
}

impl From<reqwest::Error> for Response {
    fn from(value: reqwest::Error) -> Self {
        let mut e = ErrNo::OTHER;
        if value.is_status() {
            e = ErrNo::STATUS;
        } else if value.is_connect() {
            e = ErrNo::CONNECT;
        } else if value.is_decode() {
            e = ErrNo::DECODE;
        }
        Response {
            error_no: e,
            error_message: value.to_string(),
            result: Bytes::from(""),
            headers: HashMap::new(),
        }
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"{{"error_no": "{}","error_message": "{}","result": "{}"}}"#,
            self.error_no.to_string(),
            self.error_message,
            String::from_utf8_lossy(&self.result[..])
        )
    }
}

impl Response {
    pub fn new(error_no: ErrNo, error_message: String, result: String) -> Self {
        Self {
            error_no,
            error_message,
            result: Bytes::from(result),
            headers: HashMap::new(),
        }
    }
    /// 生成一个空的成功的`Response`对象
    pub fn blank_success() -> Self {
        Self::new(ErrNo::SUCCESS, "".to_string(), "".to_string())
    }
}

type Data = Value;

/// 请求封装类
impl Request {
    /// 从传入的`headers`参数生成`reqwest::blocking::ClientBuilder`
    fn get_builder_with_headers(
        headers: Option<&HashMap<String, String>>,
    ) -> reqwest::ClientBuilder {
        let mut builder = reqwest::ClientBuilder::new();
        if let Some(headers) = headers {
            let mut header = HeaderMap::new();
            for (k, v) in headers {
                header.insert(
                    HeaderName::from_str(k).unwrap(),
                    HeaderValue::from_str(v).unwrap(),
                );
            }
            builder = builder.default_headers(header);
        }
        builder
    }
    /// send Head request
    /// # Examples
    /// ```
    /// use qcos::request::Request;
    /// use std::collections::HashMap;
    /// async {
    /// let mut headers = HashMap::new();
    /// headers.insert("x-test-header".to_string(), "test-header".to_string());
    /// Request::head("https://www.baiduc.com", None, Some(&headers)).await;
    /// };
    /// ```
    pub async fn head(
        url: &str,
        query: Option<&HashMap<String, String>>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<Response, Response> {
        Request::do_req(
            Method::Head,
            url,
            query,
            headers,
            None,
            None,
            None as Option<Body>,
        )
        .await
    }
    /// send get request
    /// # Examples
    /// ```
    /// use qcos::request::Request;
    /// use std::collections::HashMap;
    /// async {
    /// let mut headers = HashMap::new();
    /// headers.insert("x-test-header".to_string(), "test-header".to_string());
    /// Request::get("https://www.baiduc.com", None, Some(&headers)).await;
    /// };
    /// ```
    pub async fn get(
        url: &str,
        query: Option<&HashMap<String, String>>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<Response, Response> {
        Request::do_req(
            Method::Get,
            url,
            query,
            headers,
            None,
            None,
            None as Option<Body>,
        )
        .await
    }
    /// send post request
    /// # Examples
    /// ```
    /// use reqwest::Body;
    /// use qcos::request::Request;
    /// use std::collections::HashMap;
    /// use serde_json::json;
    /// async {
    /// let mut form = HashMap::new();
    /// form.insert("hello", json!(1i16));
    /// form.insert("hello1", json!("world"));
    /// let mut json = HashMap::new();
    /// json.insert("hello", json!(1i64));
    /// json.insert("hello_json", json!("world"));
    /// json.insert("data", json!(vec![1u8, 2u8, 3u8] as Vec<u8>));
    /// let resp = Request::post(
    ///     "https://www.baidu.com",
    ///     None,
    ///     None,
    ///     Some(&form),
    ///     Some(&json),
    ///     None as Option<Body>,
    /// ).await;
    /// };
    /// ```
    pub async fn post<T: Into<Body>>(
        url: &str,
        query: Option<&HashMap<String, String>>,
        headers: Option<&HashMap<String, String>>,
        form: Option<&HashMap<&str, Data>>,
        json: Option<&HashMap<&str, Data>>,
        body_data: Option<T>,
    ) -> Result<Response, Response> {
        Request::do_req(Method::Post, url, query, headers, form, json, body_data).await
    }

    /// send put request
    pub async fn put<T: Into<Body>>(
        url: &str,
        query: Option<&HashMap<String, String>>,
        headers: Option<&HashMap<String, String>>,
        form: Option<&HashMap<&str, Data>>,
        json: Option<&HashMap<&str, Data>>,
        body_data: Option<T>,
    ) -> Result<Response, Response> {
        Request::do_req(Method::Put, url, query, headers, form, json, body_data).await
    }

    /// send delete request
    pub async fn delete(
        url: &str,
        query: Option<&HashMap<String, String>>,
        headers: Option<&HashMap<String, String>>,
        form: Option<&HashMap<&str, Data>>,
        json: Option<&HashMap<&str, Data>>,
    ) -> Result<Response, Response> {
        Request::do_req(
            Method::Delete,
            url,
            query,
            headers,
            form,
            json,
            None as Option<Body>,
        )
        .await
    }

    async fn do_req<T: Into<Body>>(
        method: Method,
        url: &str,
        query: Option<&HashMap<String, String>>,
        headers: Option<&HashMap<String, String>>,
        form: Option<&HashMap<&str, Data>>,
        json: Option<&HashMap<&str, Data>>,
        body_data: Option<T>,
    ) -> Result<Response, Response> {
        let builder = Self::get_builder_with_headers(headers);
        let client = builder.timeout(Duration::from_secs(24 * 3600)).build()?;
        let mut req = match method {
            Method::Get => client.get(url),
            Method::Delete => client.delete(url),
            Method::Post => client.post(url),
            Method::Put => client.put(url),
            Method::Head => client.head(url),
        };
        if let Some(v) = query {
            req = req.query(v);
        }
        if let Some(v) = form {
            req = req.form(v);
        }
        if let Some(v) = json {
            req = req.json(v);
        }
        if let Some(v) = body_data {
            req = req.body(v.into());
        }
        let resp = req.send().await?;
        let status_code = resp.status();
        let mut error_no = ErrNo::SUCCESS;
        let mut message = "".to_string();
        if status_code.is_client_error() || status_code.is_server_error() {
            error_no = ErrNo::STATUS;
            message = format!("{}", status_code);
        }
        let mut headers = HashMap::new();
        for (k, v) in resp.headers() {
            headers.insert(k.to_string(), String::from_utf8_lossy(v.as_bytes()).into());
        }
        Ok(Response {
            error_no,
            error_message: message,
            result: resp.bytes().await?,
            headers,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::request::{ErrNo, Request};
    use reqwest::Body;
    use serde_json::json;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_get() {
        let mut header = HashMap::new();
        header.insert("user-agent".to_string(), "test-user-agent".to_string());
        let mut query = HashMap::new();
        query.insert("a".to_string(), "a".to_string());
        query.insert("b".to_string(), "b".to_string());
        query.insert("c".to_string(), "c".to_string());
        let response = Request::get("https://www.baidu.com", Some(&query), Some(&header)).await;
        match response {
            Ok(e) => {
                println!("{:#?}", e);
            }
            Err(e) => println!("{}", e),
        }
    }

    #[tokio::test]
    async fn test_post_form() {
        let mut form = HashMap::new();
        form.insert("hello", json!(1i16));
        form.insert("hello1", json!("world"));
        let mut json = HashMap::new();
        json.insert("hello", json!(1i64));
        json.insert("hello_json", json!("world"));
        json.insert("data", json!(vec![1u8, 2u8, 3u8] as Vec<u8>));
        let resp = Request::post(
            "https://www.baidu.com",
            None,
            None,
            Some(&form),
            Some(&json),
            None as Option<Body>,
        )
        .await;
        if let Ok(e) = &resp {
            println!("{:#?}", e);
        }
        if let Err(e) = resp {
            assert_eq!(e.error_no, ErrNo::DECODE)
        }
    }
}
