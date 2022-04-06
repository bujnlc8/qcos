//! object操作相关
use crate::acl;
use crate::client;
pub use crate::request::{
    CompleteMultipartUpload, ErrNo, InitiateMultipartUploadResult, Part, Request, Response,
};
pub use mime;
pub use quick_xml::de::from_str;
pub use quick_xml::se::to_string;
pub use reqwest::blocking::Body;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::io::Cursor;
use std::io::SeekFrom;

/// 分块上传阈值
const PART_THRESHOLD: usize = 1024 * 1024 * 1024 * 5;
/// 分块上传大小 默认100M
const PART_SIZE: usize = 1024 * 1024 * 100;

pub trait Objects {
    /// 上传本地文件
    fn put_object(
        &self,
        file: std::fs::File,
        key: &str,
        content_type: mime::Mime,
        acl_header: Option<&acl::AclHeader>,
        is_part: bool,
    ) -> Response;

    /// 上传二进制流
    fn put_object_binary<T: Into<Body>>(
        &self,
        file: T,
        key: &str,
        content_type: mime::Mime,
        acl_header: Option<&acl::AclHeader>,
    ) -> Response;

    /// 删除文件
    fn delete_object(&self, key: &str) -> Response;

    /// 获取文件二进制流
    fn get_object_binary(&self, key: &str) -> Response;

    /// 下载文件到本地
    fn get_object(&self, key: &str, file_name: &str) -> Response;

    /// 获取分块上传的upload_id
    fn put_object_get_upload_id(
        &self,
        key: &str,
        content_type: &mime::Mime,
        acl_header: Option<&acl::AclHeader>,
    ) -> Response;

    /// 分块上传
    fn put_objet_part(
        &self,
        key: &str,
        upload_id: &str,
        part_number: usize,
        body: Vec<u8>,
        content_type: &mime::Mime,
        acl_header: Option<&acl::AclHeader>,
    ) -> Response;

    /// 完成分块上传
    fn put_object_complete_part(
        &self,
        key: &str,
        etag_map: &HashMap<i32, String>,
        upload_id: &str,
    ) -> Response;

    /// Abort Multipart Upload 用来实现舍弃一个分块上传并删除已上传的块。
    /// 当您调用 Abort Multipart Upload 时，如果有正在使用这个 Upload Parts 上传块的请求，
    /// 则 Upload Parts 会返回失败。当该 UploadId 不存在时，会返回404 NoSuchUpload。
    fn abort_object_part(&self, key: &str, upload_id: &str) -> Response;
}

impl<'a> Objects for client::Client<'a> {
    /// 上传本地文件
    /// 见[官网文档](https://cloud.tencent.com/document/product/436/7749)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// use mime;
    /// use qcos::acl::{AclHeader, ObjectAcl};
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_object_x_cos_acl(ObjectAcl::AuthenticatedRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let file = std::fs::File::open("Cargo.toml").unwrap();
    /// let res = client.put_object(file, "Cargo.toml", mime::TEXT_PLAIN_UTF_8, Some(&acl_header), false);
    /// assert!(res.error_message.contains("403"));
    /// let file = std::fs::File::open("Cargo.toml").unwrap();
    /// // 分块传输
    /// let res = client.put_object(file, "Cargo.toml", mime::TEXT_PLAIN_UTF_8, Some(&acl_header), true);
    /// assert!(res.error_message.contains("403"));
    /// ```
    fn put_object(
        &self,
        file: std::fs::File,
        key: &str,
        content_type: mime::Mime,
        acl_header: Option<&acl::AclHeader>,
        is_part: bool,
    ) -> Response {
        // 设置为分块上传或者大于5G会启动分块上传
        let file_size = file.metadata().unwrap().len() as usize;
        let mut file = file;
        if is_part || file_size > PART_THRESHOLD {
            let mut part_number = 1;
            let mut start;
            let mut etag_map = HashMap::new();
            let upload_id = self.put_object_get_upload_id(key, &content_type, acl_header);
            if upload_id.error_no != ErrNo::SUCCESS {
                return upload_id;
            }
            let upload_id = String::from_utf8_lossy(&upload_id.result[..]).to_string();
            loop {
                start = PART_SIZE * (part_number - 1);
                if start >= file_size {
                    // 调用合并
                    let resp = self.put_object_complete_part(key, &etag_map, upload_id.as_str());
                    if resp.error_no != ErrNo::SUCCESS {
                        // 调用清理
                        self.abort_object_part(key, upload_id.as_str());
                    }
                    return resp;
                }
                let mut size = PART_SIZE;
                if file_size - start < PART_SIZE {
                    size = file_size - start;
                }
                file.seek(SeekFrom::Start(start as u64)).unwrap();
                let mut body: Vec<u8> = vec![0; size as usize];
                //file.take(size as u64).read(&mut body).unwrap();
                file.read_exact(&mut body).unwrap();
                let resp = self.put_objet_part(
                    key,
                    upload_id.as_str(),
                    part_number,
                    body,
                    &content_type,
                    acl_header,
                );
                if resp.error_no != ErrNo::SUCCESS {
                    // 调用清理
                    self.abort_object_part(key, upload_id.as_str());
                    return resp;
                }
                etag_map.insert(part_number as i32, resp.headers["etag"].clone());
                part_number += 1;
            }
        }
        let mut headers = self.gen_common_headers();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        headers.insert("Content-Length".to_string(), file_size.to_string());
        let url_path = self.get_path_from_object_key(key);
        headers =
            self.get_headers_with_auth("put", url_path.as_str(), acl_header, Some(headers), None);
        let resp = Request::put(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
            None,
            Some(&headers),
            None,
            None,
            Some(file),
        );
        match resp {
            Ok(e) => e,
            Err(e) => e,
        }
    }

    /// 上传二进制流
    /// 见[官网文档](https://cloud.tencent.com/document/product/436/7749)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// use mime;
    /// use qcos::acl::{AclHeader, ObjectAcl};
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_object_x_cos_acl(ObjectAcl::AuthenticatedRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let buffer = std::fs::read("Cargo.toml").unwrap();
    /// let res = client.put_object_binary(buffer, "Cargo.toml", mime::TEXT_PLAIN_UTF_8, Some(&acl_header));
    /// assert!(res.error_message.contains("403"));
    /// ```
    fn put_object_binary<T: Into<Body>>(
        &self,
        file: T,
        key: &str,
        content_type: mime::Mime,
        acl_header: Option<&acl::AclHeader>,
    ) -> Response {
        let mut body: Body = file.into();
        let file_size = body.buffer().unwrap().len();
        let mut headers = self.gen_common_headers();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        headers.insert("Content-Length".to_string(), file_size.to_string());
        let url_path = self.get_path_from_object_key(key);
        headers =
            self.get_headers_with_auth("put", url_path.as_str(), acl_header, Some(headers), None);
        let resp = Request::put(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
            None,
            Some(&headers),
            None,
            None,
            Some(body),
        );
        match resp {
            Ok(e) => e,
            Err(e) => e,
        }
    }
    /// 删除文件
    /// 见[官网文档](https://cloud.tencent.com/document/product/436/7743)
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
        let headers = self.get_headers_with_auth("delete", url_path.as_str(), None, None, None);
        let resp = Request::delete(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
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

    /// 下载文件二进制流
    /// 见[官网文档](https://cloud.tencent.com/document/product/436/7753)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.get_object_binary("Cargo.toml");
    /// assert!(res.error_message.contains("403"));
    /// ```
    fn get_object_binary(&self, key: &str) -> Response {
        let url_path = self.get_path_from_object_key(key);
        let headers = self.get_headers_with_auth("get", url_path.as_str(), None, None, None);
        let resp = Request::get(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
            None,
            Some(&headers),
        );
        match resp {
            Ok(e) => e,
            Err(e) => e,
        }
    }

    /// 下载文件到本地
    /// 见[官网文档](https://cloud.tencent.com/document/product/436/7753)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.get_object("Cargo.toml", "Cargo.toml");
    /// assert!(res.error_message.contains("403"));
    /// ```
    fn get_object(&self, key: &str, file_name: &str) -> Response {
        let resp = self.get_object_binary(key);
        if resp.error_no == ErrNo::SUCCESS {
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
            if let Err(e) = std::io::copy(&mut Cursor::new(resp.result), &mut output_file) {
                return Response::new(ErrNo::OTHER, format!("下载文件失败: {}", e), "".to_string());
            }
            return Response::blank_success();
        }
        resp
    }
    /// 请求实现初始化分块上传，成功执行此请求后将返回 UploadId，用于后续的 Upload Part 请求
    /// [官网文档](https://cloud.tencent.com/document/product/436/7746)
    fn put_object_get_upload_id(
        &self,
        key: &str,
        content_type: &mime::Mime,
        acl_header: Option<&acl::AclHeader>,
    ) -> Response {
        let mut query = HashMap::new();
        query.insert("uploads".to_string(), "".to_string());
        let url_path = self.get_path_from_object_key(key);
        let mut headers = self.gen_common_headers();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        let headers = self.get_headers_with_auth(
            "post",
            url_path.as_str(),
            acl_header,
            Some(headers),
            Some(&query),
        );
        let resp = Request::post(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
            Some(&query),
            Some(&headers),
            None,
            None,
            None as Option<Body>,
        );
        match resp {
            Ok(e) => {
                if e.error_no != ErrNo::SUCCESS {
                    return e;
                }
                let res = String::from_utf8_lossy(&e.result[..]);
                let res: InitiateMultipartUploadResult = from_str(res.as_ref()).unwrap();
                Response::new(ErrNo::SUCCESS, "".to_string(), res.upload_id)
            }
            Err(e) => e,
        }
    }

    /// 分块上传文件
    /// [官网文档](https://cloud.tencent.com/document/product/436/7750)
    fn put_objet_part(
        &self,
        key: &str,
        upload_id: &str,
        part_number: usize,
        body: Vec<u8>,
        content_type: &mime::Mime,
        acl_header: Option<&acl::AclHeader>,
    ) -> Response {
        let mut headers = self.gen_common_headers();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        headers.insert("Content-Length".to_string(), body.len().to_string());
        let url_path = self.get_path_from_object_key(key);
        let mut query = HashMap::new();
        query.insert("partNumber".to_string(), part_number.to_string());
        query.insert("uploadId".to_string(), upload_id.to_string());
        headers = self.get_headers_with_auth(
            "put",
            url_path.as_str(),
            acl_header,
            Some(headers),
            Some(&query),
        );
        let resp = Request::put(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
            Some(&query),
            Some(&headers),
            None,
            None,
            Some(body),
        );
        match resp {
            Ok(e) => e,
            Err(e) => e,
        }
    }

    /// 完成分块上传
    /// [官网文档](https://cloud.tencent.com/document/product/436/7742)
    fn put_object_complete_part(
        &self,
        key: &str,
        etag_map: &HashMap<i32, String>,
        upload_id: &str,
    ) -> Response {
        let url_path = self.get_path_from_object_key(key);
        let mut query = HashMap::new();
        query.insert("uploadId".to_string(), upload_id.to_string());
        let mut headers = self.gen_common_headers();
        headers.insert("Content-Type".to_string(), "application/xml".to_string());
        let headers = self.get_headers_with_auth(
            "post",
            url_path.as_str(),
            None,
            Some(headers),
            Some(&query),
        );
        let mut parts = Vec::new();
        // 按part_number排序
        let mut keys = Vec::new();
        for k in etag_map.keys() {
            keys.push(k);
        }
        keys.sort();
        for k in keys {
            parts.push(Part {
                part_number: k.clone(),
                etag: etag_map[&k].clone(),
            })
        }
        let complete = CompleteMultipartUpload { part: parts };
        let serialized_str = to_string(&complete).unwrap();
        let resp = Request::post(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
            Some(&query),
            Some(&headers),
            None,
            None,
            Some(serialized_str),
        );
        match resp {
            Ok(e) => e,
            Err(e) => e,
        }
    }

    /// 终止分块上传，清理文件碎片
    /// [官网文档](https://cloud.tencent.com/document/product/436/7740)
    fn abort_object_part(&self, key: &str, upload_id: &str) -> Response {
        let url_path = self.get_path_from_object_key(key);
        let mut query = HashMap::new();
        query.insert("uploadId".to_string(), upload_id.to_string());
        let headers =
            self.get_headers_with_auth("delete", url_path.as_str(), None, None, Some(&query));
        let resp = Request::delete(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
            Some(&query),
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
