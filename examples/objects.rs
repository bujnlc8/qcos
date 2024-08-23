//! 上传文件

use std::path::PathBuf;

use qcos::acl::{AclHeader, ObjectAcl};
use qcos::client::Client;
use qcos::objects::mime;
use qcos::request::ErrNo;

#[tokio::main]
async fn main() {
    let client = Client::new(
        "your secrect id",
        "your secrect key",
        "bucket name",
        "region",
    );
    // 普通上传，无权限控制
    let file_path = PathBuf::from("Cargo.toml");
    let res = client
        .put_object(&file_path, "Cargo.toml", Some(mime::TEXT_PLAIN_UTF_8), None)
        .await;
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 私有权限控制
    let mut acl = AclHeader::new();
    acl.insert_object_x_cos_acl(ObjectAcl::PRIVATE);
    let res = client
        .put_object(
            &file_path,
            "Cargo.toml",
            Some(mime::TEXT_PLAIN_UTF_8),
            Some(acl),
        )
        .await;
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 分块上传
    let res = client
        .clone()
        .put_big_object(
            &file_path,
            "Cargo.toml",
            Some(mime::TEXT_PLAIN_UTF_8),
            Some(qcos::objects::StorageClassEnum::ARCHIVE),
            None,
            Some(1024 * 1024),
            None,
        )
        .await;
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 分块上传，带进度条
    #[cfg(feature = "progress-bar")]
    let res = client
        .clone()
        .put_big_object_progress_bar(
            &file_path,
            "Cargo.toml",
            Some(mime::TEXT_PLAIN_UTF_8),
            Some(qcos::objects::StorageClassEnum::ARCHIVE),
            None,
            Some(1024 * 1024),
            None,
            None,
        )
        .await;
    #[cfg(feature = "progress-bar")]
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 直接上传二进制数据
    let res = client
        .put_object_binary(
            std::fs::read(&file_path).unwrap(),
            "Cargo.toml",
            Some(mime::TEXT_PLAIN_UTF_8),
            None,
        )
        .await;
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 删除文件 test/Cargo.toml
    let res = client.delete_object("test/Cargo.toml").await;
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 将对象存储对象名称为Cargo.toml的文件下载到本地，名称为local_Cargo.toml
    let res = client
        .get_object("Cargo.toml", "local_Cargo.toml", Some(10))
        .await;
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }

    // 将对象存储对象名称为Cargo.toml的文件下载到本地，名称为local_Cargo.toml.1，并显示下载进度条
    #[cfg(feature = "progress-bar")]
    let res = client
        .get_object_progress_bar("Cargo.toml", "local_Cargo.toml", Some(10), None)
        .await;
    #[cfg(feature = "progress-bar")]
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 获取预签名下载URL
    let url = client.get_presigned_download_url("Cargo.toml", 3600);
    println!("full_url: {}", url);
}
