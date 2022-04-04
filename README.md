[![qcos](https://github.com/bujnlc8/qcos/actions/workflows/qcos.yml/badge.svg?branch=master)](https://github.com/bujnlc8/qcos/actions/workflows/qcos.yml)

本包提供腾讯云对象存储(cos) 基本的操作，包括`bucket`创建及删除，对象的上传、下载、删除等。后续有时间会补充其他接口的实现。

# How to use

```rust
use qcos::client::Client;
use qcos::objects::Objects;
use qcos::acl::{AclHeader, ObjectAcl};
use qcos::request::ErrorNo;
use mime;

fn main(){
    let client = Client::new("Your secrect id", "Your secrect key", "bucket name", "region");
    let mut acl_header = AclHeader::new();
    acl_header.insert_object_x_cos_acl(ObjectAcl::AuthenticatedRead);
    let res = client.put_object("Cargo.toml", "Cargo.toml", mime::TEXT_PLAIN_UTF_8, Some(acl_header));
    if res.error_no == ErrorNo::SUCCESS{
        println!("success");
      } else{
        println!("{}", res.error_message);
    }
}

```
如果操作成功，会打印出`success`, 否则会打印出失败原因。


# Installation

insert into your project's cargo.toml block next line

```
[dependencies]
qcos = "0.0.1"

```
