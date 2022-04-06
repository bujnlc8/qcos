//! bucket相关接口

use crate::client::Client;

use crate::request::{Request, Response};
use reqwest::blocking::Body;

use crate::acl::AclHeader;
use std::collections::HashMap;
pub trait Bucket {
    /// 在指定账号下创建一个存储桶
    /// 创建存储桶时，如果没有指定访问权限，则默认使用私有读写（private）权限。
    fn put_bucket(&self, acl_header: Option<&AclHeader>) -> Response;

    /// 用于删除指定的存储桶。该 API 的请求者需要对存储桶有写入权限。
    fn delete_bucket(&self) -> Response;

    /// 可以列出该存储桶内的部分或者全部对象。该 API 的请求者需要对存储桶有读取权限。
    fn list_objects(
        &self,
        prefix: &str,
        delimiter: &str,
        encoding_type: &str,
        marker: &str,
        max_keys: i32,
    ) -> Response;
}

impl<'a> Bucket for Client<'a> {
    /// 创建一个存储桶
    /// 见[官网文档](https://cloud.tencent.com/document/product/436/7738)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::bucket::Bucket;
    /// use qcos::acl::{AclHeader, BucketAcl};
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_bucket_x_cos_acl(BucketAcl::PublicRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.put_bucket(Some(&acl_header));
    /// assert!(res.error_message.contains("403"));
    /// ```
    fn put_bucket(&self, acl_header: Option<&AclHeader>) -> Response {
        let headers = self.get_headers_with_auth("put", "/", acl_header, None, None);
        let resp = Request::put(
            self.get_full_url_from_path("/").as_str(),
            None,
            Some(&headers),
            None,
            None,
            None as Option<Body>,
        );
        self.make_response(resp)
    }
    /// 删除指定的存储桶。该 API 的请求者需要对存储桶有写入权限。
    /// 见[官网文档](https://cloud.tencent.com/document/product/436/7732)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::bucket::Bucket;
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.delete_bucket();
    /// assert!(res.error_message.contains("403"));
    /// ```
    fn delete_bucket(&self) -> Response {
        let headers = self.get_headers_with_auth("delete", "/", None, None, None);
        let resp = Request::delete(
            self.get_full_url_from_path("/").as_str(),
            None,
            Some(&headers),
            None,
            None,
        );
        self.make_response(resp)
    }
    /// 列出该存储桶内的部分或者全部对象。该 API 的请求者需要对存储桶有读取权限。
    /// 见[官网文档](https://cloud.tencent.com/document/product/436/7734)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::bucket::Bucket;
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.list_objects("prefix", "", "", "/", 100);
    /// assert!(res.error_message.contains("403"));
    /// ```
    fn list_objects(
        &self,
        prefix: &str,
        delimiter: &str,
        encoding_type: &str,
        marker: &str,
        max_keys: i32,
    ) -> Response {
        let mut query = HashMap::new();
        if prefix.len() > 0 {
            query.insert("prefix".to_string(), prefix.to_string());
        }
        if delimiter.len() > 0 {
            query.insert("delimiter".to_string(), delimiter.to_string());
        }
        if encoding_type.len() > 0 {
            query.insert("encoding-type".to_string(), encoding_type.to_string());
        }
        if marker.len() > 0 {
            query.insert("marker".to_string(), marker.to_string());
        }
        if max_keys <= 1000 && max_keys > 0 {
            query.insert("max-keys".to_string(), max_keys.to_string());
        }
        let headers = self.get_headers_with_auth("get", "/", None, None, Some(&query));
        let resp = Request::get(
            self.get_full_url_from_path("/").as_str(),
            Some(&query),
            Some(&headers),
        );
        self.make_response(resp)
    }
}
