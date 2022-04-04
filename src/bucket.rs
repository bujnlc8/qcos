//! bucket相关接口

use crate::client::Client;

use crate::request::{Request, Response};
use crate::signer;

use crate::acl::AclHeader;
pub trait Bucket {
    fn put_bucket(&self, acl_header: Option<AclHeader>) -> Response;
    fn delete_bucket(&self) -> Response;
}

impl<'a> Bucket for Client<'a> {
    /// 创建一个存储桶
    /// 见[文档](https://cloud.tencent.com/document/product/436/7738)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::bucket::Bucket;
    /// use qcos::acl::{AclHeader, BucketAcl};
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_bucket_x_cos_acl(BucketAcl::PublicRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.put_bucket(Some(acl_header));
    /// assert!(res.error_message.contains("403"));
    /// ```
    fn put_bucket(&self, acl_header: Option<AclHeader>) -> Response {
        let mut headers = self.gen_common_headers();
        if let Some(acl_header) = acl_header {
            for (k, v) in acl_header.get_headers() {
                headers.insert(k.to_string(), v.to_string());
            }
        }
        let signature = signer::Signer::new("put", "/", Some(&headers), None).get_signature(
            self.get_secrect_key(),
            self.get_secrect_id(),
            7200,
        );
        headers.insert("Authorization".to_string(), signature);
        let resp = Request::put(
            self.get_full_url_from_path("/").as_str(),
            None,
            Some(&headers),
            None,
            None,
            None,
        );
        match resp {
            Ok(e) => e,
            Err(e) => e,
        }
    }
    /// 删除指定的存储桶。该 API 的请求者需要对存储桶有写入权限。
    /// 见[文档](https://cloud.tencent.com/document/product/436/7732)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::bucket::Bucket;
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.delete_bucket();
    /// assert!(res.error_message.contains("403"));
    /// ```
    fn delete_bucket(&self) -> Response {
        let mut headers = self.gen_common_headers();
        let signature = signer::Signer::new("delete", "/", Some(&headers), None).get_signature(
            self.get_secrect_key(),
            self.get_secrect_id(),
            7200,
        );
        headers.insert("Authorization".to_string(), signature);
        let resp = Request::delete(
            self.get_full_url_from_path("/").as_str(),
            None,
            Some(&headers),
            None,
            None,
        );
        match resp {
            Ok(e) => e,
            Err(e) => e,
        }
    }
}
