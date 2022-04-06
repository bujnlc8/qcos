//! 上传文件

use qcos::acl::{AclHeader, ObjectAcl};
use qcos::client::Client;
use qcos::objects::{mime, Objects};
use qcos::request::ErrNo;

fn main() {
    let client = Client::new(
        "Your secrect id",
        "Your secrect key",
        "bucket name",
        "region",
    );
    let file = std::fs::File::open("Cargo.toml").unwrap();
    // 普通上传，无权限控制
    let res = client.put_object(file, "Cargo.toml", mime::TEXT_PLAIN_UTF_8, None, false);
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 私有权限控制
    let mut acl = AclHeader::new();
    acl.insert_object_x_cos_acl(ObjectAcl::PRIVATE);
    let file = std::fs::File::open("Cargo.toml").unwrap();
    let res = client.put_object(
        file,
        "Cargo.toml",
        mime::TEXT_PLAIN_UTF_8,
        Some(&acl),
        false,
    );
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 分块传输，当文件大于5GB时，自动采用分块策略
    let file = std::fs::File::open("Cargo.toml").unwrap();
    let res = client.put_object(file, "Cargo.toml", mime::TEXT_PLAIN_UTF_8, None, true);
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 直接上传二进制流
    let res = client.put_object_binary(
        std::fs::read("Cargo.toml").unwrap(),
        "Cargo.toml",
        mime::TEXT_PLAIN_UTF_8,
        None,
    );
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 删除文件 test/Cargo.toml
    let res = client.delete_object("test/Cargo.toml");
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 将对象存储对象名称为Cargo.toml的文件下载到本地，名称为local_Cargo.toml
    let res = client.get_object("Cargo.toml", "local_Cargo.toml");
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
}
