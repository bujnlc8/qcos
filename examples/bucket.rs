//! bucket管理

use qcos::acl::{AclHeader, BucketAcl};
use qcos::bucket::Bucket;
use qcos::client::Client;
use qcos::request::ErrNo;
use qcos::service::Service;

fn main() {
    let client = Client::new(
        "Your secrect id",
        "Your secrect key",
        "bucket-name",
        "region",
    );
    // 获取bukcet列表
    let res = client.get_bucket_list();
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{:?}", res.result);
    }
    // 删除bucket
    let res = client.delete_bucket();
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{:?}", res.result);
    }
    // 创建bucket(无权限控制), 创建的bucket即上初始化传入的bucket-name
    let res = client.put_bucket(None);
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{:?}", res.result);
    }
    // 创建bucket(有权限控制)
    let mut acl = AclHeader::new();
    acl.insert_bucket_x_cos_acl(BucketAcl::PRIVATE);
    let res = client.put_bucket(Some(&acl));
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{:?}", res.result);
    }
    // 列出key以`abc`开头的文件
    let res = client.list_objects("abc", "", "", "", 0);
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{:?}", res.result);
    }
}
