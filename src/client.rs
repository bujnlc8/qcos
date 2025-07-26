//! 接口客户端，所有的操作都基于该对象
use crate::acl::AclHeader;
use crate::request::Response;
use crate::signer::Signer;

/// 接口请求Client
/// # Examples
/// ```
/// use qcos::client::Client;
/// let client = Client::new("secrect_id", "secrect_key", "bucket", "region");
/// assert_eq!(client.get_host(), "bucket.cos.region.myqcloud.com");
///```
use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, DATE, HOST};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone)]
pub struct Client {
    secrect_id: String,
    secrect_key: String,
    bucket: String,
    region: String,
}

impl Client {
    pub fn new(
        secrect_id: impl Into<String>,
        secrect_key: impl Into<String>,
        bucket: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        Self {
            secrect_id: secrect_id.into(),
            secrect_key: secrect_key.into(),
            bucket: bucket.into(),
            region: region.into(),
        }
    }

    pub fn get_host(&self) -> String {
        format!("{}.cos.{}.myqcloud.com", self.bucket, self.region)
    }

    pub fn get_secrect_key(&self) -> &str {
        &self.secrect_key
    }
    pub fn get_secrect_id(&self) -> &str {
        &self.secrect_id
    }

    // 生成通用的request headers, 包含`Host`及`Date`
    pub fn get_common_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(HOST, HeaderValue::from_str(&self.get_host()).unwrap());
        let now_str = Utc::now().format("%a, %d %b %Y %T GMT").to_string();
        headers.insert(DATE, HeaderValue::from_str(&now_str).unwrap());
        headers
    }

    pub fn get_full_url_from_path(&self, path: &str) -> String {
        format!("https://{}{}", self.get_host(), path)
    }

    pub fn get_path_from_object_key(&self, key: &str) -> String {
        let mut url_path = key.to_string();
        if !url_path.starts_with('/') {
            url_path = format!("/{}", url_path);
        }
        url_path
    }
    // 生成查询bucket list的host
    pub fn get_host_for_bucket_query(&self) -> String {
        if self.region.is_empty() {
            return "service.cos.myqcloud.com".to_string();
        }
        format!("cos.{}.myqcloud.com", self.region)
    }

    // 返回带有`Authorization` 的headers, 如果headers从参数传入, 除添加acl头部之外不会添加其他头
    // 否则以`gen_common_headers` 返回作为初始值
    pub fn get_headers_with_auth(
        &self,
        method: &str,
        url_path: &str,
        acl_header: Option<AclHeader>,
        origin_headers: Option<HeaderMap>,
        query: Option<HashMap<String, String>>,
    ) -> HeaderMap {
        let mut headers = match origin_headers {
            Some(header) => header,
            None => self.get_common_headers(),
        };
        if let Some(acl_header) = acl_header {
            for (k, v) in acl_header.get_headers() {
                headers.insert(
                    HeaderName::from_str(k).unwrap(),
                    HeaderValue::from_str(v).unwrap(),
                );
            }
        }
        let signature = Signer::new(method, url_path, Some(headers.clone()), query).get_signature(
            self.get_secrect_key(),
            self.get_secrect_id(),
            7200,
        );
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&signature).unwrap());
        headers
    }

    pub fn make_response(&self, resp: Result<Response, Response>) -> Response {
        resp.unwrap_or_else(|x| x)
    }

    /// 获取预签名下载URL
    /// <https://cloud.tencent.com/document/product/436/35153>
    pub fn get_presigned_download_url(&self, object_key: &str, expire: u32) -> String {
        let url_path = self.get_path_from_object_key(object_key);
        let full_url = self.get_full_url_from_path(url_path.as_str());
        let mut headers = HeaderMap::new();
        headers.insert(HOST, HeaderValue::from_str(&self.get_host()).unwrap());
        let signature = Signer::new("get", &url_path, Some(headers), None).get_signature(
            self.get_secrect_key(),
            self.get_secrect_id(),
            expire,
        );
        format!("{url}?{signature}", url = full_url, signature = signature)
    }

    /// 获取web直传签名
    /// <https://cloud.tencent.com/document/product/436/9067>
    pub fn get_upload_signature(
        &self,
        object_key: &str,
        acl_header: Option<AclHeader>,
        origin_headers: Option<HeaderMap>,
    ) -> String {
        let url_path = self.get_path_from_object_key(object_key);
        let header = self.get_headers_with_auth("put", &url_path, acl_header, origin_headers, None);
        header
            .get(AUTHORIZATION)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}
