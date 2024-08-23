//! bucket相关接口 方法见 [`crate::client::Client`#impl-Client]

use crate::client::Client;

use crate::request::{Request, Response};
use reqwest::Body;

use crate::acl::AclHeader;
use std::collections::HashMap;

// 为了兼容以前的版本
pub struct Bucket;

impl Client {
    /// 创建一个存储桶
    /// <https://cloud.tencent.com/document/product/436/7738>
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::acl::{AclHeader, BucketAcl};
    /// async {
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_bucket_x_cos_acl(BucketAcl::PublicRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
    /// let res = client.put_bucket(Some(acl_header)).await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    pub async fn put_bucket(&self, acl_header: Option<AclHeader>) -> Response {
        let headers = self.get_headers_with_auth("put", "/", acl_header, None, None);
        let resp = Request::put(
            self.get_full_url_from_path("/").as_str(),
            None,
            Some(&headers),
            None,
            None,
            None as Option<Body>,
        )
        .await;
        self.make_response(resp)
    }
    /// 删除指定的存储桶。该 API 的请求者需要对存储桶有写入权限。
    /// <https://cloud.tencent.com/document/product/436/7732>
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// async {
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
    /// let res = client.delete_bucket().await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    pub async fn delete_bucket(&self) -> Response {
        let headers = self.get_headers_with_auth("delete", "/", None, None, None);
        let resp = Request::delete(
            self.get_full_url_from_path("/").as_str(),
            None,
            Some(&headers),
            None,
            None,
        )
        .await;
        self.make_response(resp)
    }
    /// 列出该存储桶内的部分或者全部对象。该 API 的请求者需要对存储桶有读取权限。
    /// <https://cloud.tencent.com/document/product/436/7734>
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// async {
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
    /// let res = client.list_objects("prefix", "", "", "/", 100).await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    pub async fn list_objects(
        &self,
        prefix: &str,
        delimiter: &str,
        encoding_type: &str,
        marker: &str,
        max_keys: i32,
    ) -> Response {
        let mut query = HashMap::new();
        if !prefix.is_empty() {
            query.insert("prefix".to_string(), prefix.to_string());
        }
        if !delimiter.is_empty() {
            query.insert("delimiter".to_string(), delimiter.to_string());
        }
        if !encoding_type.is_empty() {
            query.insert("encoding-type".to_string(), encoding_type.to_string());
        }
        if !marker.is_empty() {
            query.insert("marker".to_string(), marker.to_string());
        }
        if max_keys <= 1000 && max_keys > 0 {
            query.insert("max-keys".to_string(), max_keys.to_string());
        }
        let headers = self.get_headers_with_auth("get", "/", None, None, Some(query.clone()));
        let resp = Request::get(
            self.get_full_url_from_path("/").as_str(),
            Some(&query),
            Some(&headers),
        )
        .await;
        self.make_response(resp)
    }

    /// 确认该存储桶是否存在，是否有权限访问
    /// <https://cloud.tencent.com/document/product/436/7735>
    /// 存储桶存在且有读取权限，返回 `SUCCESS`
    /// 无存储桶读取权限，返回 `ErrNo::STATUS`, error_message包含403。
    /// 存储桶不存在，返回 `ErrNo::STATUS`, error_message包含404。
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// async {
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
    /// let res = client.check_bucket().await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    pub async fn check_bucket(&self) -> Response {
        let headers = self.get_headers_with_auth("head", "/", None, None, None);
        let resp = Request::head(
            self.get_full_url_from_path("/").as_str(),
            None,
            Some(&headers),
        )
        .await;
        self.make_response(resp)
    }
    /// 写入存储桶的访问控制列表
    /// <https://cloud.tencent.com/document/product/436/7737>
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::acl::{AclHeader, BucketAcl};
    /// async {
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_bucket_x_cos_acl(BucketAcl::PublicRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
    /// let res = client.put_bucket(Some(acl_header)).await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    pub async fn put_bucket_acl(&self, acl_header: AclHeader) -> Response {
        let mut query = HashMap::new();
        query.insert("acl".to_string(), String::new());
        let headers =
            self.get_headers_with_auth("put", "/", Some(acl_header), None, Some(query.clone()));
        let resp = Request::put(
            self.get_full_url_from_path("/").as_str(),
            Some(&query),
            Some(&headers),
            None,
            None,
            None as Option<Body>,
        )
        .await;
        self.make_response(resp)
    }
}
