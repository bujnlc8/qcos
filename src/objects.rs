//! object操作相关
use crate::acl;
use crate::client;
pub use crate::request::{ErrNo, Request, Response};
use crate::signer;
pub use mime;
use std::fs;
use std::io::Cursor;

pub trait Objects {
    fn put_object(
        &self,
        path: &str,
        key: &str,
        content_type: mime::Mime,
        acl_header: Option<acl::AclHeader>,
    ) -> Response;
    fn delete_object(&self, key: &str) -> Response;
    fn get_object(&self, key: &str, file_name: &str) -> Response;
}

impl<'a> Objects for client::Client<'a> {
    /// 上传文件
    /// 见[文档](https://cloud.tencent.com/document/product/436/7749)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// use mime;
    /// use qcos::acl::{AclHeader, ObjectAcl};
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_object_x_cos_acl(ObjectAcl::AuthenticatedRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.put_object("Cargo.toml", "Cargo.toml", mime::TEXT_PLAIN_UTF_8, Some(acl_header));
    /// assert!(res.error_message.contains("403"));
    /// ```
    fn put_object(
        &self,
        path: &str,
        key: &str,
        content_type: mime::Mime,
        acl_header: Option<acl::AclHeader>,
    ) -> Response {
        let metadata_r = fs::metadata(path);
        let metadata;
        match metadata_r {
            Ok(e) => metadata = e,
            Err(e) => {
                return Response::new(
                    ErrNo::OTHER,
                    format!("读取metadata失败: {}", e),
                    "".to_string(),
                );
            }
        }
        let content_r = fs::read(path);
        let content;
        match content_r {
            Ok(e) => content = e,
            Err(e) => {
                return Response::new(
                    ErrNo::OTHER,
                    format!("读取文件内容失败: {}", e),
                    "".to_string(),
                );
            }
        }
        let content_length = metadata.len().to_string();
        let mut headers = self.gen_common_headers();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        headers.insert("Content-Length".to_string(), content_length);
        if let Some(acl_header) = acl_header {
            for (k, v) in acl_header.get_headers() {
                headers.insert(k.to_string(), v.to_string());
            }
        }
        let url_path = self.get_path_from_object_key(key);
        let signature = signer::Signer::new("put", url_path.as_str(), Some(&headers), None)
            .get_signature(self.get_secrect_key(), self.get_secrect_id(), 7200);
        headers.insert("Authorization".to_string(), signature);
        let resp = Request::put(
            self.get_full_url_from_path(&url_path).as_str(),
            None,
            Some(&headers),
            None,
            None,
            Some(content),
        );
        match resp {
            Ok(e) => e,
            Err(e) => e,
        }
    }

    /// 删除文件
    /// 见[文档](https://cloud.tencent.com/document/product/436/7743)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.delete_object("Cargo.toml");
    /// assert!(res.error_message.contains("403"));
    /// ```
    fn delete_object(&self, key: &str) -> Response {
        let url_path = self.get_path_from_object_key(key);
        let mut headers = self.gen_common_headers();
        let signature = signer::Signer::new("delete", url_path.as_str(), Some(&headers), None)
            .get_signature(self.get_secrect_key(), self.get_secrect_id(), 7200);
        headers.insert("Authorization".to_string(), signature);
        let resp = Request::delete(
            self.get_full_url_from_path(&url_path).as_str(),
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

    /// 下载文件
    /// 见[文档](https://cloud.tencent.com/document/product/436/7753)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.get_object("Cargo.toml", "Cargo.toml");
    /// assert!(res.error_message.contains("403"));
    /// ```
    fn get_object(&self, key: &str, file_name: &str) -> Response {
        let mut headers = self.gen_common_headers();
        let url_path = self.get_path_from_object_key(key);
        let signature = signer::Signer::new("get", url_path.as_str(), Some(&headers), None)
            .get_signature(self.get_secrect_key(), self.get_secrect_id(), 7200);
        headers.insert("Authorization".to_string(), signature);
        let resp = Request::get(
            self.get_full_url_from_path(&url_path).as_str(),
            None,
            Some(&headers),
        );
        match resp {
            Ok(e) => {
                if e.error_no == ErrNo::SUCCESS {
                    let output_file_r = fs::File::create(file_name);
                    let mut output_file;
                    match output_file_r {
                        Ok(e) => output_file = e,
                        Err(e) => {
                            return Response::new(
                                ErrNo::OTHER,
                                format!("创建文件失败: {}", e),
                                "".to_string(),
                            );
                        }
                    }
                    if let Err(e) = std::io::copy(&mut Cursor::new(e.result), &mut output_file) {
                        return Response::new(
                            ErrNo::OTHER,
                            format!("下载文件失败: {}", e),
                            "".to_string(),
                        );
                    }
                    return Response::blank_success();
                }
                e
            }
            Err(e) => e,
        }
    }
}
