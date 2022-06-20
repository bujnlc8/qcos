//! bucket管理

use qcos::acl::{AclHeader, BucketAcl};
use qcos::bucket::Bucket;
use qcos::client::Client;
use qcos::request::ErrNo;
use qcos::service::Service;

#[tokio::main]
async fn main() {
    let client = Client::new(
        "Your secrect id".to_owned(),
        "Your secrect key.to_owned()",
        "bucket-name".to_owned(),
        "region".to_owned(),
    );
    // 获取bukcet列表
    let res = client.get_bucket_list().await;
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{:?}", res.result);
    }
    // 删除bucket
    let res = client.delete_bucket().await;
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{:?}", res.result);
    }
    // 创建bucket(无权限控制), 创建的bucket即上初始化传入的bucket-name
    let res = client.put_bucket(None).await;
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{:?}", res.result);
    }
    // 创建bucket(有权限控制)
    let mut acl = AclHeader::new();
    acl.insert_bucket_x_cos_acl(BucketAcl::PRIVATE);
    let res = client.put_bucket(Some(&acl)).await;
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{:?}", res.result);
    }
    // 列出key以`abc`开头的文件
    let res = client.list_objects("abc", "", "", "", 0).await;
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{:?}", res.result);
    }
    // 检查bucket状态
    let res = client.check_bucket().await;
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
    // 写入存储桶的访问控制列表（ACL）
    let mut acl_header = AclHeader::new();
    acl_header.insert_bucket_x_cos_acl(BucketAcl::PRIVATE);
    let res = client.put_bucket_acl(&acl_header).await;
    if res.error_no == ErrNo::SUCCESS {
        println!("SUCCESS");
    } else {
        println!("{}", res.error_message);
    }
}
