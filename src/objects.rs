//! object操作相关
#![allow(clippy::too_many_arguments)]

use crate::acl;
use crate::client;
pub use crate::request::{
    CompleteMultipartUpload, ErrNo, InitiateMultipartUploadResult, Part, Request, Response,
};
pub use mime;
pub use quick_xml::de::from_str;
pub use quick_xml::se::to_string;
pub use reqwest::Body;
use std::io::Cursor;
use std::{collections::HashMap, path::PathBuf};
use tokio::{fs, io::copy};

// 最小上传分片大小 1MB
const PART_MIN_SIZE: u64 = 1024 * 1024;

// 最大上传分片大小1GB
const PART_MAX_SIZE: u64 = 1024 * 1024 * 1024;

/// 存储类型
/// <https://cloud.tencent.com/document/product/436/33417>
pub enum StorageClassEnum {
    MazStandard,
    MazStandardIa,
    IntelligentTiering,
    MazIntelligentTiering,
    StandardIa,
    ARCHIVE,
    DeepArchive,
    STANDARD,
}

impl From<StorageClassEnum> for String {
    fn from(value: StorageClassEnum) -> Self {
        match value {
            StorageClassEnum::ARCHIVE => String::from("ARCHIVE"),
            StorageClassEnum::STANDARD => String::from("STANDARD"),
            StorageClassEnum::StandardIa => String::from("STANDARD_IA"),
            StorageClassEnum::MazStandard => String::from("MAZ_STANDARD"),
            StorageClassEnum::DeepArchive => String::from("DEEP_ARCHIVE"),
            StorageClassEnum::MazStandardIa => String::from("MAZ_STANDARD_IA"),
            StorageClassEnum::IntelligentTiering => String::from("INTELLIGENT_TIERING"),
            StorageClassEnum::MazIntelligentTiering => String::from("MAZ_INTELLIGENT_TIERING"),
        }
    }
}

#[async_trait::async_trait]
pub trait Objects {
    /// 上传本地小文件
    async fn put_object(
        &self,
        file_path: &str,
        key: &str,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
    ) -> Response;

    /// 上传本地大文件
    async fn put_big_object(
        self,
        file_path: &str,
        key: &str,
        content_type: Option<mime::Mime>,
        storage_class: Option<StorageClassEnum>,
        acl_header: Option<acl::AclHeader>,
        part_size: Option<u64>,
        max_threads: Option<u64>,
    ) -> Response;

    /// 上传二进制流
    async fn put_object_binary<T: Into<Body> + Send>(
        &self,
        file: T,
        key: &str,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
    ) -> Response;

    /// 删除文件
    async fn delete_object(&self, key: &str) -> Response;

    /// 获取文件二进制流
    async fn get_object_binary(&self, key: &str) -> Response;

    /// 下载文件到本地
    async fn get_object(&self, key: &str, file_name: &str) -> Response;

    /// 获取分块上传的upload_id
    async fn put_object_get_upload_id(
        &self,
        key: &str,
        content_type: Option<mime::Mime>,
        storage_class: Option<StorageClassEnum>,
        acl_header: Option<acl::AclHeader>,
    ) -> Response;

    /// 分块上传
    async fn put_object_part(
        self,
        key: &str,
        upload_id: &str,
        part_number: u64,
        body: Vec<u8>,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
    ) -> Response;

    /// 完成分块上传
    async fn put_object_complete_part(
        &self,
        key: &str,
        etag_map: &HashMap<u64, String>,
        upload_id: &str,
    ) -> Response;

    /// Abort Multipart Upload 用来实现舍弃一个分块上传并删除已上传的块。
    /// 当您调用 Abort Multipart Upload 时，如果有正在使用这个 Upload Parts 上传块的请求，
    /// 则 Upload Parts 会返回失败。当该 UploadId 不存在时，会返回404 NoSuchUpload。
    async fn abort_object_part(&self, key: &str, upload_id: &str) -> Response;
}

#[async_trait::async_trait]
impl Objects for client::Client {
    /// 上传本地小文件
    /// 见[官网文档](https://cloud.tencent.com/document/product/436/7749)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// use mime;
    /// use qcos::acl::{AclHeader, ObjectAcl};
    /// async {
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_object_x_cos_acl(ObjectAcl::AuthenticatedRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.put_object("Cargo.toml", "Cargo.toml", Some(mime::TEXT_PLAIN_UTF_8), Some(acl_header)).await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    async fn put_object(
        &self,
        file_path: &str,
        key: &str,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
    ) -> Response {
        let file = match tokio::fs::File::open(file_path).await {
            Ok(file) => file,
            Err(e) => {
                return Response::new(
                    ErrNo::IO,
                    format!("打开文件失败: {}, {}", file_path, e),
                    Default::default(),
                )
            }
        };
        // 设置为分块上传或者大于5G会启动分块上传
        let file_size = match file.metadata().await {
            Ok(meta) => meta.len() as usize,
            Err(e) => {
                return Response::new(
                    ErrNo::IO,
                    format!("获取文件大小失败: {}, {}", file_path, e),
                    Default::default(),
                )
            }
        };
        let mut headers = self.gen_common_headers();
        headers.insert(
            "Content-Type".to_string(),
            content_type
                .unwrap_or(mime::APPLICATION_OCTET_STREAM)
                .to_string(),
        );
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
        )
        .await;
        self.make_response(resp)
    }

    /// 上传本地大文件
    /// 见[官网文档](https://cloud.tencent.com/document/product/436/7749)
    /// # 参数
    /// file_path: 文件路径
    /// key: 上传文件的key，如test/Cargo.lock
    /// content_type: 文件类型
    /// storage_class: 存储类型`StorageClassEnum` 默认STANDARD
    /// acl_header: 请求控制
    /// part_size: 分片大小，单位bytes，要求1M-1G之间，默认100M
    /// max_threads: 最大上传线程数，默认20
    ///
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::{Objects, StorageClassEnum};
    /// use mime;
    /// use qcos::acl::{AclHeader, ObjectAcl};
    /// async {
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_object_x_cos_acl(ObjectAcl::AuthenticatedRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// // 分块传输
    /// let res = client.put_big_object("Cargo.toml","Cargo.toml", Some(mime::TEXT_PLAIN_UTF_8), Some(StorageClassEnum::STANDARD), Some(acl_header), Some(1024 * 1024 * 100), None).await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    async fn put_big_object(
        self,
        file_path: &str,
        key: &str,
        content_type: Option<mime::Mime>,
        storage_class: Option<StorageClassEnum>,
        acl_header: Option<acl::AclHeader>,
        part_size: Option<u64>,
        max_threads: Option<u64>,
    ) -> Response {
        use tokio::io::AsyncReadExt;
        let part_size = part_size.unwrap_or(PART_MAX_SIZE / 10);
        assert!((PART_MIN_SIZE..PART_MAX_SIZE).contains(&part_size));
        assert!(max_threads.unwrap_or(20) <= 1000);
        let mut file = match tokio::fs::File::open(file_path).await {
            Ok(file) => file,
            Err(e) => {
                return Response::new(
                    ErrNo::IO,
                    format!("打开文件失败: {}, {}", file_path, e),
                    Default::default(),
                )
            }
        };
        // 设置为分块上传或者大于5G会启动分块上传
        let file_size = match file.metadata().await {
            Ok(meta) => meta.len(),
            Err(e) => {
                return Response::new(
                    ErrNo::IO,
                    format!("获取文件大小失败: {}, {}", file_path, e),
                    Default::default(),
                )
            }
        };
        let mut part_number = 1;
        let mut etag_map = HashMap::new();
        let upload_id = self
            .put_object_get_upload_id(key, content_type.clone(), storage_class, acl_header.clone())
            .await;
        if upload_id.error_no != ErrNo::SUCCESS {
            return upload_id;
        }
        let upload_id = String::from_utf8_lossy(&upload_id.result[..]).to_string();
        let max_threads = max_threads.unwrap_or(20);
        let mut tasks = Vec::new();
        let mut upload_bytes = 0;
        let mut part_number1 = 1;
        loop {
            if upload_bytes >= file_size {
                break;
            }
            let mut part_size1 = part_size;
            let last_bytes = file_size - upload_bytes;
            // 倒数第二次上传后剩余小于1M，附加到倒数第二次上传
            if last_bytes < part_size + PART_MIN_SIZE && last_bytes < PART_MAX_SIZE {
                part_size1 = last_bytes;
            }
            let mut body: Vec<u8> = vec![0; part_size1 as usize];
            if let Err(e) = file.read_exact(&mut body).await {
                // 调用清理
                self.abort_object_part(key, &upload_id).await;
                return Response::new(
                    ErrNo::IO,
                    format!("读取文件失败: {}, {}", file_path, e),
                    Default::default(),
                );
            }
            upload_bytes += part_size1;
            if tasks.len() < max_threads as usize {
                let key = key.to_string();
                let upload_id = upload_id.clone();
                let this = self.clone();
                let acl_header = acl_header.clone();
                let content_type = content_type.clone();
                let handle = tokio::spawn(async move {
                    let resp = this
                        .clone()
                        .put_object_part(
                            &key,
                            &upload_id,
                            part_number,
                            body,
                            content_type,
                            acl_header,
                        )
                        .await;
                    if resp.error_no != ErrNo::SUCCESS {
                        // 调用清理
                        this.abort_object_part(&key, upload_id.as_str()).await;
                    }
                    resp
                });
                tasks.push(handle);
                part_number += 1;
            } else {
                for task in tasks {
                    let response = task.await.unwrap();
                    if response.error_no != ErrNo::SUCCESS {
                        return response;
                    }
                    etag_map.insert(part_number1, response.headers["etag"].clone());
                    part_number1 += 1;
                }
                tasks = Vec::new();
            }
        }
        if !tasks.is_empty() {
            for task in tasks {
                let response = task.await.unwrap();
                if response.error_no != ErrNo::SUCCESS {
                    return response;
                }
                etag_map.insert(part_number1, response.headers["etag"].clone());
                part_number1 += 1;
            }
        }
        // 调用合并
        let resp = self
            .put_object_complete_part(key, &etag_map, upload_id.as_str())
            .await;
        if resp.error_no != ErrNo::SUCCESS {
            // 调用清理
            self.abort_object_part(key, upload_id.as_str()).await;
        }
        return resp;
    }

    /// 上传二进制流
    /// 见[官网文档](https://cloud.tencent.com/document/product/436/7749)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// use mime;
    /// use qcos::acl::{AclHeader, ObjectAcl};
    /// async {
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_object_x_cos_acl(ObjectAcl::AuthenticatedRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let buffer = std::fs::read("Cargo.toml").unwrap();
    /// let res = client.put_object_binary(buffer, "Cargo.toml", Some(mime::TEXT_PLAIN_UTF_8), Some(acl_header)).await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    async fn put_object_binary<T: Into<Body> + Send>(
        &self,
        file: T,
        key: &str,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
    ) -> Response {
        let body: Body = file.into();
        let bytes = body.as_bytes();
        if bytes.is_none() {
            return Response::new(ErrNo::IO, "不是内存对象".to_owned(), Default::default());
        }
        let file_size = bytes.unwrap().len();
        let mut headers = self.gen_common_headers();
        headers.insert(
            "Content-Type".to_string(),
            content_type
                .unwrap_or(mime::APPLICATION_OCTET_STREAM)
                .to_string(),
        );
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
        )
        .await;
        self.make_response(resp)
    }
    /// 删除文件
    /// 见[官网文档](https://cloud.tencent.com/document/product/436/7743)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// async {
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.delete_object("Cargo.toml").await;
    /// assert!(res.error_message.contains("403"))
    /// };
    /// ```
    async fn delete_object(&self, key: &str) -> Response {
        let url_path = self.get_path_from_object_key(key);
        let headers = self.get_headers_with_auth("delete", url_path.as_str(), None, None, None);
        let resp = Request::delete(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
            None,
            Some(&headers),
            None,
            None,
        )
        .await;
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
    /// async {
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.get_object_binary("Cargo.toml").await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    async fn get_object_binary(&self, key: &str) -> Response {
        let url_path = self.get_path_from_object_key(key);
        let headers = self.get_headers_with_auth("get", url_path.as_str(), None, None, None);
        let resp = Request::get(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
            None,
            Some(&headers),
        )
        .await;
        self.make_response(resp)
    }

    /// 下载文件到本地
    /// 见[官网文档](https://cloud.tencent.com/document/product/436/7753)
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// async {
    /// let client = Client::new("foo", "bar", "qcloudtest-1256650966", "ap-guangzhou");
    /// let res = client.get_object("Cargo.toml", "Cargo.toml").await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    async fn get_object(&self, key: &str, file_name: &str) -> Response {
        let resp = self.get_object_binary(key).await;
        if resp.error_no == ErrNo::SUCCESS {
            let file_path = PathBuf::from(file_name);
            if let Some(parent_file_path) = file_path.parent() {
                if !parent_file_path.exists() {
                    fs::create_dir_all(parent_file_path).await.unwrap();
                }
            }
            let mut output_file;
            match fs::File::create(file_name).await {
                Ok(e) => output_file = e,
                Err(e) => {
                    return Response::new(
                        ErrNo::OTHER,
                        format!("创建文件失败: {}", e),
                        "".to_string(),
                    );
                }
            }
            if let Err(e) = copy(&mut Cursor::new(resp.result), &mut output_file).await {
                return Response::new(ErrNo::OTHER, format!("下载文件失败: {}", e), "".to_string());
            }
            return Response::blank_success();
        }
        resp
    }
    /// 请求实现初始化分块上传，成功执行此请求后将返回 UploadId，用于后续的 Upload Part 请求
    /// [官网文档](https://cloud.tencent.com/document/product/436/7746)
    async fn put_object_get_upload_id(
        &self,
        key: &str,
        content_type: Option<mime::Mime>,
        storage_class: Option<StorageClassEnum>,
        acl_header: Option<acl::AclHeader>,
    ) -> Response {
        let mut query = HashMap::new();
        query.insert("uploads".to_string(), "".to_string());
        let url_path = self.get_path_from_object_key(key);
        let mut headers = self.gen_common_headers();
        headers.insert(
            "Content-Type".to_string(),
            content_type
                .unwrap_or(mime::APPLICATION_OCTET_STREAM)
                .to_string(),
        );
        headers.insert(
            "x-cos-storage-class".to_string(),
            storage_class.unwrap_or(StorageClassEnum::STANDARD).into(),
        );
        let headers = self.get_headers_with_auth(
            "post",
            url_path.as_str(),
            acl_header,
            Some(headers),
            Some(query.clone()),
        );
        let resp = Request::post(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
            Some(&query),
            Some(&headers),
            None,
            None,
            None as Option<Body>,
        )
        .await;
        match resp {
            Ok(res) => {
                if res.error_no != ErrNo::SUCCESS {
                    return res;
                }
                match quick_xml::de::from_reader::<&[u8], InitiateMultipartUploadResult>(
                    &res.result[..],
                ) {
                    Ok(res) => Response::new(ErrNo::SUCCESS, "".to_string(), res.upload_id),
                    Err(e) => Response::new(ErrNo::DECODE, e.to_string(), Default::default()),
                }
            }
            Err(e) => e,
        }
    }

    /// 分块上传文件
    /// [官网文档](https://cloud.tencent.com/document/product/436/7750)
    async fn put_object_part(
        self,
        key: &str,
        upload_id: &str,
        part_number: u64,
        body: Vec<u8>,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
    ) -> Response {
        let mut headers = self.gen_common_headers();
        headers.insert(
            "Content-Type".to_string(),
            content_type
                .unwrap_or(mime::APPLICATION_OCTET_STREAM)
                .to_string(),
        );
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
            Some(query.clone()),
        );
        let resp = Request::put(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
            Some(&query),
            Some(&headers),
            None,
            None,
            Some(body),
        )
        .await;
        self.make_response(resp)
    }

    /// 完成分块上传
    /// [官网文档](https://cloud.tencent.com/document/product/436/7742)
    async fn put_object_complete_part(
        &self,
        key: &str,
        etag_map: &HashMap<u64, String>,
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
            Some(query.clone()),
        );
        let mut parts = Vec::new();
        // 按part_number排序
        let mut etag_map_tuple: Vec<(&u64, &String)> = etag_map.iter().collect();
        etag_map_tuple.sort_by(|a, b| a.0.cmp(b.0));
        for (k, v) in etag_map_tuple {
            parts.push(Part {
                part_number: *k,
                etag: v.to_string(),
            })
        }
        let complete = CompleteMultipartUpload { part: parts };
        let serialized_str = match to_string(&complete) {
            Ok(s) => s,
            Err(e) => return Response::new(ErrNo::ENCODE, e.to_string(), Default::default()),
        };
        let resp = Request::post(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
            Some(&query),
            Some(&headers),
            None,
            None,
            Some(serialized_str),
        )
        .await;
        self.make_response(resp)
    }

    /// 终止分块上传，清理文件碎片
    /// [官网文档](https://cloud.tencent.com/document/product/436/7740)
    async fn abort_object_part(&self, key: &str, upload_id: &str) -> Response {
        let url_path = self.get_path_from_object_key(key);
        let mut query = HashMap::new();
        query.insert("uploadId".to_string(), upload_id.to_string());
        let headers = self.get_headers_with_auth(
            "delete",
            url_path.as_str(),
            None,
            None,
            Some(query.clone()),
        );
        let resp = Request::delete(
            self.get_full_url_from_path(url_path.as_str()).as_str(),
            Some(&query),
            Some(&headers),
            None,
            None,
        )
        .await;
        self.make_response(resp)
    }
}
