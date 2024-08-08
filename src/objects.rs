//! object操作相关
#![allow(clippy::too_many_arguments)]

use crate::acl;
use crate::client;
pub use crate::request::{
    CompleteMultipartUpload, ErrNo, InitiateMultipartUploadResult, Part, Request, Response,
};
#[cfg(feature = "progress-bar")]
use futures_util::TryStreamExt;
#[cfg(feature = "progress-bar")]
pub use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
pub use mime;
pub use quick_xml::de::from_str;
pub use quick_xml::se::to_string;
use reqwest::header::{HeaderName, HeaderValue, RANGE};
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
pub use reqwest::Body;
use std::io::Cursor;
use std::{collections::HashMap, path::PathBuf};
use tokio::io::AsyncReadExt;
use tokio::{fs, io::copy};
#[cfg(feature = "progress-bar")]
use tokio_util::io::ReaderStream;

use std::str::FromStr;
#[cfg(feature = "progress-bar")]
use tokio::io::BufReader;

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

    /// 上传本地小文件，带进度条
    #[cfg(feature = "progress-bar")]
    async fn put_object_progress_bar(
        &self,
        file_path: &str,
        key: &str,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
        progress_style: Option<ProgressStyle>,
    ) -> Response;

    /// 上传本地大文件，带进度条
    #[cfg(feature = "progress-bar")]
    async fn put_big_object_progress_bar(
        self,
        file_path: &str,
        key: &str,
        content_type: Option<mime::Mime>,
        storage_class: Option<StorageClassEnum>,
        acl_header: Option<acl::AclHeader>,
        part_size: Option<u64>,
        max_threads: Option<u64>,
        progress_style: Option<ProgressStyle>,
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

    /// 上传二进制数据，带进度条
    #[cfg(feature = "progress-bar")]
    async fn put_object_binary_progress_bar<
        T: Into<Body> + Send + Sync + tokio::io::AsyncRead + 'static,
    >(
        &self,
        file: T,
        key: &str,
        file_size: u64,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
        progress_style: Option<ProgressStyle>,
    ) -> Response;

    /// 上传二进制数据
    async fn put_object_binary<T: Into<Body> + Send>(
        &self,
        file: T,
        key: &str,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
    ) -> Response;

    /// 删除文件
    async fn delete_object(&self, key: &str) -> Response;

    /// 获取文件二进制数据
    #[cfg(feature = "progress-bar")]
    async fn get_object_binary_progress_bar(
        &self,
        key: &str,
        threads: Option<u8>,
        progress_style: Option<ProgressStyle>,
    ) -> Response;

    /// 获取文件二进制数据，多线程模式
    async fn get_object_binary(&self, key: &str, threads: Option<u8>) -> Response;

    /// 下载文件到本地
    async fn get_object(&self, key: &str, file_name: &str, threads: Option<u8>) -> Response;

    async fn get_object_size(&self, key: &str) -> usize;

    /// 下载文件到本地
    #[cfg(feature = "progress-bar")]
    async fn get_object_progress_bar(
        &self,
        key: &str,
        file_name: &str,
        threads: Option<u8>,
        progress_style: Option<ProgressStyle>,
    ) -> Response;

    /// 获取分块上传的upload_id
    async fn put_object_get_upload_id(
        &self,
        key: &str,
        content_type: Option<mime::Mime>,
        storage_class: Option<StorageClassEnum>,
        acl_header: Option<acl::AclHeader>,
    ) -> Response;

    /// 分块上传，带进度条
    #[cfg(feature = "progress-bar")]
    async fn put_object_part_progress_bar(
        self,
        key: &str,
        upload_id: &str,
        part_number: u64,
        body: Vec<u8>,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
        pb: ProgressBar,
    ) -> Response;

    /// 分块上传文件
    /// <https://cloud.tencent.com/document/product/436/7750>
    async fn put_object_part<T: Into<Body> + Send>(
        self,
        key: &str,
        upload_id: &str,
        part_number: u64,
        body: T,
        file_size: u64,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
    ) -> Response;

    /// 完成分块上传
    async fn put_object_complete_part(
        &self,
        key: &str,
        etag_map: HashMap<u64, String>,
        upload_id: &str,
    ) -> Response;

    /// Abort Multipart Upload 用来实现舍弃一个分块上传并删除已上传的块。
    /// 当您调用 Abort Multipart Upload 时，如果有正在使用这个 Upload Parts 上传块的请求，
    /// 则 Upload Parts 会返回失败。当该 UploadId 不存在时，会返回404 NoSuchUpload。
    async fn abort_object_part(&self, key: &str, upload_id: &str) -> Response;
}

#[async_trait::async_trait]
impl Objects for client::Client {
    /// 上传本地小文件，无进度条
    /// <https://cloud.tencent.com/document/product/436/7749>
    /// # 参数
    /// - file_path: 文件路径
    /// - key: 上传文件的key，如test/Cargo.lock
    /// - content_type: 文件类型
    /// - acl_header: 请求控制
    ///
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// use mime;
    /// use qcos::acl::{AclHeader, ObjectAcl};
    /// async {
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_object_x_cos_acl(ObjectAcl::AuthenticatedRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
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
        self.put_object_binary(file, key, content_type, acl_header)
            .await
    }

    /// 上传本地小文件，带进度条
    /// <https://cloud.tencent.com/document/product/436/7749>
    /// # 参数
    /// - file_path: 文件路径
    /// - key: 上传文件的key，如test/Cargo.lock
    /// - content_type: 文件类型
    /// - acl_header: 请求控制
    /// - progress_style: 进度条样式
    ///
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// use mime;
    /// use qcos::acl::{AclHeader, ObjectAcl};
    /// async {
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_object_x_cos_acl(ObjectAcl::AuthenticatedRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
    /// let res = client.put_object_progress_bar("Cargo.toml", "Cargo.toml", Some(mime::TEXT_PLAIN_UTF_8), Some(acl_header), None).await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    #[cfg(feature = "progress-bar")]
    async fn put_object_progress_bar(
        &self,
        file_path: &str,
        key: &str,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
        progress_style: Option<ProgressStyle>,
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
        let file_size = file.metadata().await.unwrap().len();
        self.put_object_binary_progress_bar(
            file,
            key,
            file_size,
            content_type,
            acl_header,
            progress_style,
        )
        .await
    }

    /// 上传本地大文件，带进度条
    /// <https://cloud.tencent.com/document/product/436/7749>
    /// # 参数
    /// - file_path: 文件路径
    /// - key: 上传文件的key，如test/Cargo.lock
    /// - content_type: 文件类型
    /// - storage_class: 存储类型`StorageClassEnum` 默认STANDARD
    /// - acl_header: 请求控制
    /// - part_size: 分片大小，单位bytes，要求1M-1G之间，默认50M
    /// - max_threads: 最大上传线程数，默认20
    /// - progress_style: 进度条样式
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
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
    /// // 分块传输
    /// let res = client.put_big_object_progress_bar("Cargo.toml","Cargo.toml", Some(mime::TEXT_PLAIN_UTF_8), Some(StorageClassEnum::STANDARD), Some(acl_header), Some(1024 * 1024 * 100), None, None).await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    #[cfg(feature = "progress-bar")]
    async fn put_big_object_progress_bar(
        self,
        file_path: &str,
        key: &str,
        content_type: Option<mime::Mime>,
        storage_class: Option<StorageClassEnum>,
        acl_header: Option<acl::AclHeader>,
        part_size: Option<u64>,
        max_threads: Option<u64>,
        progress_style: Option<ProgressStyle>,
    ) -> Response {
        let part_size = part_size.unwrap_or(PART_MAX_SIZE / 10 / 2);
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
        let upload_id_response = self
            .put_object_get_upload_id(key, content_type.clone(), storage_class, acl_header.clone())
            .await;
        if upload_id_response.error_no != ErrNo::SUCCESS {
            return upload_id_response;
        }
        let upload_id = String::from_utf8_lossy(&upload_id_response.result[..]).to_string();
        // 默认20个线程
        let max_threads = max_threads.unwrap_or(20);
        let mut tasks = Vec::new();
        let mut upload_bytes = 0;
        let mut part_number1 = 1;
        let multi = MultiProgress::new();
        let sty = match progress_style{
            Some(sty)=>sty,
            None=> ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap().progress_chars("#>-")};
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
                let pb = multi.add(ProgressBar::new(body.len() as u64));
                pb.set_style(sty.clone());
                let handle = tokio::spawn(async move {
                    let resp = this
                        .clone()
                        .put_object_part_progress_bar(
                            &key,
                            &upload_id,
                            part_number,
                            body,
                            content_type,
                            acl_header,
                            pb,
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
            .put_object_complete_part(key, etag_map, upload_id.as_str())
            .await;
        if resp.error_no != ErrNo::SUCCESS {
            // 调用清理
            self.abort_object_part(key, upload_id.as_str()).await;
        }
        return resp;
    }

    /// 上传本地大文件，无进度条
    /// <https://cloud.tencent.com/document/product/436/7749>
    /// # 参数
    /// - file_path: 文件路径
    /// - key: 上传文件的key，如test/Cargo.lock
    /// - content_type: 文件类型
    /// - storage_class: 存储类型`StorageClassEnum` 默认STANDARD
    /// - acl_header: 请求控制
    /// - part_size: 分片大小，单位bytes，要求1M-1G之间，默认50M
    /// - max_threads: 最大上传线程数，默认20
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
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
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
        let part_size = part_size.unwrap_or(PART_MAX_SIZE / 10 / 2);
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
        let upload_id_response = self
            .put_object_get_upload_id(key, content_type.clone(), storage_class, acl_header.clone())
            .await;
        if upload_id_response.error_no != ErrNo::SUCCESS {
            return upload_id_response;
        }
        let upload_id = String::from_utf8_lossy(&upload_id_response.result[..]).to_string();
        // 默认20个线程
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
                            part_size1,
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
            .put_object_complete_part(key, etag_map, upload_id.as_str())
            .await;
        if resp.error_no != ErrNo::SUCCESS {
            // 调用清理
            self.abort_object_part(key, upload_id.as_str()).await;
        }
        return resp;
    }

    /// 上传二进制数据，带进度条
    /// <https://cloud.tencent.com/document/product/436/7749>
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// use mime;
    /// use qcos::acl::{AclHeader, ObjectAcl};
    /// async {
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_object_x_cos_acl(ObjectAcl::AuthenticatedRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
    /// let buffer = tokio::fs::File::open("Cargo.toml").await.unwrap();
    /// let res = client.put_object_binary_progress_bar(buffer, "Cargo.toml", 100, Some(mime::TEXT_PLAIN_UTF_8), Some(acl_header), None).await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    #[cfg(feature = "progress-bar")]
    async fn put_object_binary_progress_bar<
        T: Into<Body> + Send + Sync + tokio::io::AsyncRead + 'static,
    >(
        &self,
        file: T,
        key: &str,
        file_size: u64,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
        progress_style: Option<ProgressStyle>,
    ) -> Response {
        let reader = ReaderStream::new(file);
        let pb = ProgressBar::new(file_size);
        let sty = match progress_style {
            Some(sty)=>sty,
            None=> ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap().progress_chars("#>-")
        };
        pb.set_style(sty);
        let pb1 = pb.clone();
        let stream = reader.inspect_ok(move |chunk| {
            pb1.inc(chunk.len() as u64);
        });
        let body = Body::wrap_stream(stream);
        let resp = self
            .put_object_binary(body, key, content_type, acl_header)
            .await;
        pb.finish();
        resp
    }

    /// 上传二进制数据
    /// <https://cloud.tencent.com/document/product/436/7749>
    /// # 参数
    /// - file: 文件T:Into\<reqwest::Body\>
    /// - key: 上传文件的key，如test/Cargo.lock
    /// - content_type: 文件类型
    /// - acl_header: 请求控制
    ///
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// use mime;
    /// use qcos::acl::{AclHeader, ObjectAcl};
    /// async {
    /// let mut acl_header = AclHeader::new();
    /// acl_header.insert_object_x_cos_acl(ObjectAcl::AuthenticatedRead);
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
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
        let mut headers = self.get_common_headers();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(
                content_type
                    .unwrap_or(mime::APPLICATION_OCTET_STREAM)
                    .as_ref(),
            )
            .unwrap(),
        );
        headers.insert(CONTENT_LENGTH, HeaderValue::from(file_size));
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
    /// <https://cloud.tencent.com/document/product/436/7743>
    /// # 参数
    /// - key: 文件的key，如test/Cargo.lock
    ///
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// async {
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
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

    /// 下载文件二进制数据，有进度条
    /// <https://cloud.tencent.com/document/product/436/7753>
    /// # 参数
    /// - key: 文件的key，如test/Cargo.lock
    /// - progress_style: 进度条样式
    ///
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// async {
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
    /// let res = client.get_object_binary_progress_bar("Cargo.toml", None, None).await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    #[cfg(feature = "progress-bar")]
    async fn get_object_binary_progress_bar(
        &self,
        key: &str,
        threads: Option<u8>,
        progress_style: Option<ProgressStyle>,
    ) -> Response {
        let size = self.get_object_size(key).await;
        let multi = MultiProgress::new();
        let sty = match progress_style {
            Some(sty)=>sty,
            None=> ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap().progress_chars("#>-")
        };
        let mut threads = threads.unwrap_or(5) as usize;
        // 小于1KB只启用1个线程
        if size < 1024 {
            threads = 1;
        }
        let url_path = self.get_path_from_object_key(key);
        let headers = self.get_headers_with_auth("get", url_path.as_str(), None, None, None);
        let url = self.get_full_url_from_path(url_path.as_str());
        let part_size = size / threads;
        let mut handles = Vec::new();
        for i in 0..threads {
            let mut headers = headers.clone();
            let url = url.clone();
            let multi = multi.clone();
            let sty = sty.clone();
            let handle = tokio::spawn(async move {
                let download_size;
                // 最后一个线程下载全部
                let range = if i == threads - 1 {
                    download_size = size - (threads - 1) * part_size;
                    String::new()
                } else {
                    download_size = part_size;
                    ((i + 1) * part_size - 1).to_string()
                };
                let pb = multi.add(ProgressBar::new(download_size as u64));
                pb.set_style(sty);
                let range = format!("bytes={}-{}", i * part_size, range);
                headers.insert(RANGE, HeaderValue::from_str(&range).unwrap());
                // let resp = Request::get(&url, None, Some(&headers)).await;
                let mut resp = reqwest::Client::new()
                    .get(url)
                    .headers(headers)
                    .send()
                    .await
                    .unwrap();
                let mut data = Vec::new();
                while let Some(chunk) = resp.chunk().await.unwrap() {
                    pb.inc(chunk.len() as u64);
                    data.push(chunk);
                }
                let resp = Response::data_success(data.concat());
                pb.finish();
                resp
            });
            handles.push(handle);
        }
        let mut data = Vec::new();
        for handle in handles {
            let response = handle.await.unwrap();
            if response.error_no != ErrNo::SUCCESS {
                return response;
            }
            data.extend(response.result);
        }
        Response::data_success(data)
    }

    /// 下载文件到本地，无进度条
    /// <https://cloud.tencent.com/document/product/436/7753>
    /// # 参数
    /// - key: 要下载的文件的key，如test/Cargo.lock
    /// - file_name: 保存文件的名称，支持带目录，会自动创建
    /// - threads: 下载线程数量，默认5
    ///
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// async {
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
    /// let res = client.get_object("Cargo.toml", "Cargo.toml", None).await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    async fn get_object(&self, key: &str, file_name: &str, threads: Option<u8>) -> Response {
        let resp = self.get_object_binary(key, threads).await;
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
                    return Response::new(ErrNo::OTHER, format!("创建文件失败: {}", e), Vec::new());
                }
            }
            if let Err(e) = copy(&mut Cursor::new(resp.result), &mut output_file).await {
                return Response::new(ErrNo::OTHER, format!("下载文件失败: {}", e), Vec::new());
            }
            return Response::default();
        }
        resp
    }

    /// 下载文件到本地，带进度条
    /// <https://cloud.tencent.com/document/product/436/7753>
    /// # 参数
    /// - key: 要下载的文件的key，如test/Cargo.lock
    /// - file_name: 保存文件的名称，支持带目录，会自动创建
    /// - threads: 下载线程数量，默认5
    /// - progress_style: 进度条样式
    ///
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// async {
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
    /// let res = client.get_object_progress_bar("Cargo.toml", "Cargo.toml", None, None).await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    #[cfg(feature = "progress-bar")]
    async fn get_object_progress_bar(
        &self,
        key: &str,
        file_name: &str,
        threads: Option<u8>,
        progress_style: Option<ProgressStyle>,
    ) -> Response {
        let resp = self
            .get_object_binary_progress_bar(key, threads, progress_style)
            .await;
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
                    return Response::new(ErrNo::OTHER, format!("创建文件失败: {}", e), Vec::new());
                }
            }
            if let Err(e) = copy(&mut Cursor::new(resp.result), &mut output_file).await {
                return Response::new(ErrNo::OTHER, format!("下载文件失败: {}", e), Vec::new());
            }
            return Response::default();
        }
        resp
    }

    /// 请求实现初始化分块上传，成功执行此请求后将返回 UploadId，用于后续的 Upload Part 请求
    /// <https://cloud.tencent.com/document/product/436/7746>
    async fn put_object_get_upload_id(
        &self,
        key: &str,
        content_type: Option<mime::Mime>,
        storage_class: Option<StorageClassEnum>,
        acl_header: Option<acl::AclHeader>,
    ) -> Response {
        let mut query = HashMap::new();
        query.insert("uploads".to_string(), String::new());
        let url_path = self.get_path_from_object_key(key);
        let mut headers = self.get_common_headers();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(
                content_type
                    .unwrap_or(mime::APPLICATION_OCTET_STREAM)
                    .as_ref(),
            )
            .unwrap(),
        );
        headers.insert(
            HeaderName::from_str("x-cos-storage-class").unwrap(),
            HeaderValue::from_str(&String::from(
                storage_class.unwrap_or(StorageClassEnum::STANDARD),
            ))
            .unwrap(),
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
                    Ok(res) => Response::new(ErrNo::SUCCESS, String::new(), res.upload_id.into()),
                    Err(e) => Response::new(ErrNo::DECODE, e.to_string(), Default::default()),
                }
            }
            Err(e) => e,
        }
    }

    /// 分块上传文件，带进度条
    /// <https://cloud.tencent.com/document/product/436/7750>
    #[cfg(feature = "progress-bar")]
    async fn put_object_part_progress_bar(
        self,
        key: &str,
        upload_id: &str,
        part_number: u64,
        body: Vec<u8>,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
        pb: ProgressBar,
    ) -> Response {
        let file_size = body.len() as u64;
        let reader = ReaderStream::new(BufReader::new(Cursor::new(body)));
        let pb1 = pb.clone();
        let stream = reader.inspect_ok(move |chunk| {
            pb1.inc(chunk.len() as u64);
        });
        let body = Body::wrap_stream(stream);
        let resp = self
            .put_object_part(
                key,
                upload_id,
                part_number,
                body,
                file_size,
                content_type,
                acl_header,
            )
            .await;
        pb.finish();
        resp
    }

    /// 分块上传文件，不带进度条
    /// <https://cloud.tencent.com/document/product/436/7750>
    async fn put_object_part<T: Into<Body> + Send>(
        self,
        key: &str,
        upload_id: &str,
        part_number: u64,
        body: T,
        file_size: u64,
        content_type: Option<mime::Mime>,
        acl_header: Option<acl::AclHeader>,
    ) -> Response {
        let mut headers = self.get_common_headers();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(
                content_type
                    .unwrap_or(mime::APPLICATION_OCTET_STREAM)
                    .as_ref(),
            )
            .unwrap(),
        );
        headers.insert(CONTENT_LENGTH, HeaderValue::from(file_size));
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
        let body: Body = body.into();
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
    /// <https://cloud.tencent.com/document/product/436/7742>
    async fn put_object_complete_part(
        &self,
        key: &str,
        etag_map: HashMap<u64, String>,
        upload_id: &str,
    ) -> Response {
        let url_path = self.get_path_from_object_key(key);
        let mut query = HashMap::new();
        query.insert("uploadId".to_string(), upload_id.to_string());
        let mut headers = self.get_common_headers();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/xml").unwrap(),
        );
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
    /// <https://cloud.tencent.com/document/product/436/7740>
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

    /// 获取文件的大小
    async fn get_object_size(&self, key: &str) -> usize {
        let url_path = self.get_path_from_object_key(key);
        let url = self.get_full_url_from_path(url_path.as_str());
        let headers = self.get_headers_with_auth("head", url_path.as_str(), None, None, None);
        let response = reqwest::Client::new()
            .head(url)
            .headers(headers)
            .send()
            .await
            .unwrap();
        let size = match response.headers().get("content-length") {
            Some(v) => v.to_str().unwrap_or("0").parse().unwrap(),
            None => 0,
        };
        size
    }

    /// 多线程获取文件二进制数据
    /// <https://cloud.tencent.com/document/product/436/7753>
    /// # 参数
    /// - key: 文件的key，如test/Cargo.lock
    ///
    /// # Examples
    /// ```
    /// use qcos::client::Client;
    /// use qcos::objects::Objects;
    /// async {
    /// let client = Client::new("foo", "bar", "qcloudtest-xxx", "ap-guangzhou");
    /// let res = client.get_object_binary("Cargo.toml", None).await;
    /// assert!(res.error_message.contains("403"));
    /// };
    /// ```
    async fn get_object_binary(&self, key: &str, threads: Option<u8>) -> Response {
        let size = self.get_object_size(key).await;
        let mut threads = threads.unwrap_or(5) as usize;
        // 小于1KB只启用1个线程
        if size < 1024 {
            threads = 1;
        }
        let url_path = self.get_path_from_object_key(key);
        let headers = self.get_headers_with_auth("get", url_path.as_str(), None, None, None);
        let url = self.get_full_url_from_path(url_path.as_str());
        let part_size = size / threads;
        let mut handles = Vec::new();
        for i in 0..threads {
            let mut headers = headers.clone();
            let url = url.clone();
            let handle = tokio::spawn(async move {
                // 最后一个线程下载全部
                let range = if i == threads - 1 {
                    String::new()
                } else {
                    ((i + 1) * part_size - 1).to_string()
                };
                let range = format!("bytes={}-{}", i * part_size, range);
                headers.insert(RANGE, HeaderValue::from_str(&range).unwrap());
                Request::get(&url, None, Some(&headers)).await
            });
            handles.push(handle);
        }
        let mut data = Vec::new();
        for handle in handles {
            let response: Response = self.make_response(handle.await.unwrap());
            if response.error_no != ErrNo::SUCCESS {
                return response;
            }
            data.extend(response.result);
        }
        Response::data_success(data)
    }
}
