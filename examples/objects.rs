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
    // 普通上传，无权限控制
    let res = client.put_object("test.png", "test.png", mime::IMAGE_PNG, None, false);
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 私有权限控制
    let mut acl = AclHeader::new();
    acl.insert_object_x_cos_acl(ObjectAcl::PRIVATE);
    let res = client.put_object("test.png", "test.png", mime::IMAGE_PNG, Some(&acl), false);
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 分块传输，当文件大于5GB时，自动采用分块策略
    let res = client.put_object("test.png", "test.png", mime::IMAGE_PNG, None, true);
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 删除文件 test/test.png
    let res = client.delete_object("test/test.png");
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 将对象存储对象名称为test.png的文件下载到本地，名称为local_test.png
    let res = client.get_object("test.png", "local_test.png");
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
}
