[![Crates.io](https://img.shields.io/crates/v/qcos?style=flat-square)](https://crates.io/crates/qcos)
[![qcos](https://github.com/bujnlc8/qcos/actions/workflows/qcos.yml/badge.svg)](https://github.com/bujnlc8/qcos/actions/workflows/qcos.yml)

**异步版本** `async`/`await`

本包提供腾讯云对象存储(cos) 基本的操作，包括`bucket`创建及删除，对象的上传、下载、删除等。

上传文件支持以下特点:

- 支持文件直传，推荐 1GB 以下的文件

- 支持分块传输，设置分块大小和最大上传线程数量

- 支持显示上传进度条(需开启`progress-bar` feature)，上传方法名称加了`_progress_bar`后缀与不显示进度条的方法区分

# How to use

```rust
use qcos::acl::{AclHeader, ObjectAcl};
use qcos::client::Client;
use qcos::objects::{mime, ErrNo, Objects};

#[tokio::main]
async fn main() {
    let client = Client::new(
        "Your secrect id",
        "Your secrect key",
        "bucket name",
        "region",
    );
    let mut acl_header = AclHeader::new();
    acl_header.insert_object_x_cos_acl(ObjectAcl::PublicRead);
    let res = client.put_object("test.png", "test.png", Some(mime::IMAGE_PNG), Some(acl_header)).await;
    if res.error_no == ErrNo::SUCCESS {
        println!("success");
    } else {
        println!("{}", res.error_message);
    }
    // 分块上传，带进度条
    #[cfg(feature = "progress-bar")]
    let res = client
        .clone()
        .put_big_object_progress_bar(
            "Cargo.toml",
            "Cargo.toml",
            Some(mime::TEXT_PLAIN_UTF_8),
            Some(qcos::objects::StorageClassEnum::ARCHIVE),
            None,
            Some(1024 * 1024),
            None,
            None,
        )
        .await;
}

```

如果操作成功，会打印出`success`, 否则会打印出失败原因。

更多的例子请参考[examples](https://github.com/bujnlc8/qcos/tree/master/examples)。

# Installation

insert into your project's cargo.toml block next line

```
[dependencies]
qcos = "0.1.8"
```

如果需要开启显示进度条的方法:

```
[dependencies]
qcos = {version = "0.1.8", features=["progress-bar"]}
```
