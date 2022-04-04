//! 接口客户端，所有的操作都基于该对象
/// 接口请求Client
/// # Examples
/// ```
/// use qcos::client::Client;
/// let client = Client::new("secrect_id", "secrect_key", "bucket", "region");
/// assert_eq!(client.get_host(), "bucket.cos.region.myqcloud.com");
///```
use chrono::Utc;
use std::collections::HashMap;

pub struct Client<'a> {
    secrect_id: &'a str,
    secrect_key: &'a str,
    bucket: &'a str,
    region: &'a str,
}

impl<'a> Client<'a> {
    pub fn new(
        secrect_id: &'a str,
        secrect_key: &'a str,
        bucket: &'a str,
        region: &'a str,
    ) -> Self {
        Self {
            secrect_id,
            secrect_key,
            bucket,
            region,
        }
    }

    pub fn get_host(&self) -> String {
        format!("{}.cos.{}.myqcloud.com", self.bucket, self.region)
    }

    pub fn get_secrect_key(&self) -> &str {
        self.secrect_key
    }
    pub fn get_secrect_id(&self) -> &str {
        self.secrect_id
    }

    // 生成通用的request headers, 包含`Host`及`Date`
    pub fn gen_common_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), self.get_host());
        let now_str = Utc::now().format("%a, %d %b %Y %T GMT").to_string();
        headers.insert("Date".to_string(), now_str);
        headers
    }

    pub fn get_full_url_from_path(&self, path: &str) -> String {
        format!("https://{}{}", self.get_host(), path)
    }

    pub fn get_path_from_object_key(&self, key: &str) -> String {
        let mut url_path = key.to_string();
        if !url_path.starts_with("/") {
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
}