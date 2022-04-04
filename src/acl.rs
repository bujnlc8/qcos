//!访问控制列表（ACL）

use std::collections::HashMap;

/// 对象的预设 ACL, 见[文档](https://cloud.tencent.com/document/product/436/30752#.E9.A2.84.E8.AE.BE.E7.9A.84-acl)
#[derive(Debug, PartialEq)]
pub enum ObjectAcl {
    /// 空描述，此时根据各级目录的显式设置及存储桶的设置来确定是否允许请求（默认）
    DEFAULT,
    /// 创建者（主账号）具备 FULL_CONTROL 权限，其他人没有权限
    PRIVATE,
    /// 创建者具备 FULL_CONTROL 权限，匿名用户组具备 READ 权限
    PublicRead,
    /// 创建者具备 FULL_CONTROL 权限，认证用户组具备 READ 权限
    AuthenticatedRead,
    /// 创建者具备 FULL_CONTROL 权限，存储桶拥有者具备 READ 权限
    BucketOwnerRead,
    /// 创建者和存储桶拥有者都具备 FULL_CONTROL 权限
    BucketOwnerFullControl,
}

/// 存储桶的预设 ACL
#[derive(Debug, PartialEq)]
pub enum BucketAcl {
    /// 创建者（主账号）具备 FULL_CONTROL 权限，其他人没有权限（默认）
    PRIVATE,
    /// 创建者具备 FULL_CONTROL 权限，匿名用户组具备 READ 权限
    PublicRead,
    /// 创建者和匿名用户组都具备 FULL_CONTROL 权限，通常不建议授予此权限
    PublicReadWrite,
    /// 创建者具备 FULL_CONTROL 权限，认证用户组具备 READ 权限
    AuthenticatedRead,
}

pub struct AclHeader {
    headers: HashMap<String, String>,
}

impl AclHeader {
    pub fn new() -> AclHeader {
        let m = HashMap::new();
        AclHeader { headers: m }
    }

    pub fn get_headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// 插入object x-cos-acl
    /// 定义对象的访问控制列表（ACL）属性。枚举值请参见 ACL 概述 文档中对象的预设 ACL 部分，
    /// 例如 default，private，public-read 等，默认为 default
    /// 注意：如果您不需要进行对象 ACL 控制，请设置为 default 或者此项不进行设置，默认继承存储桶权限
    pub fn insert_object_x_cos_acl(&mut self, x_cos_acl: ObjectAcl) -> &mut Self {
        let v;
        match x_cos_acl {
            ObjectAcl::AuthenticatedRead => v = "authenticated-read",
            ObjectAcl::DEFAULT => v = "default",
            ObjectAcl::PublicRead => v = "public-read",
            ObjectAcl::PRIVATE => v = "private",
            ObjectAcl::BucketOwnerRead => v = "bucket-owner-read",
            ObjectAcl::BucketOwnerFullControl => v = "bucket-owner-full-control",
        }
        self.headers.insert("x-cos-acl".to_string(), v.to_string());
        self
    }

    /// 赋予被授权者读取对象(桶)的权限，格式为 id="\[OwnerUin\]"，
    /// 例如 id="100000000001"，可使用半角逗号（,）分隔多组被授权者，例如id="100000000001",id="100000000002"
    pub fn insert_x_cos_grant_read(&mut self, x_cos_grant_read: String) -> &mut Self {
        self.headers
            .insert("x-cos-grant-read".to_string(), x_cos_grant_read);
        self
    }

    /// 赋予被授权者读取对象(桶)的访问控制列表（ACL）的权限，格式为 id="\[OwnerUin\]"，
    /// 例如 id="100000000001"，可使用半角逗号（,）分隔多组被授权者，例如id="100000000001",id="100000000002"
    pub fn insert_x_cos_grant_read_acp(&mut self, x_cos_grant_read_acp: String) -> &mut Self {
        self.headers
            .insert("x-cos-grant-read-acp".to_string(), x_cos_grant_read_acp);
        self
    }
    /// 赋予被授权者写入对象(桶)的访问控制列表（ACL）的权限，格式为 id="\[OwnerUin\]"，
    /// 例如 id="100000000001"，可使用半角逗号（,）分隔多组被授权者，例如id="100000000001",id="100000000002"
    pub fn insert_x_cos_grant_write_acp(&mut self, x_cos_grant_write_acp: String) -> &mut Self {
        self.headers
            .insert("x-cos-grant-write-acp".to_string(), x_cos_grant_write_acp);
        self
    }
    /// 赋予被授权者操作对象(桶)的所有权限，格式为 id="\[OwnerUin\]"，
    /// 例如 id="100000000001"，可使用半角逗号（,）分隔多组被授权者，例如id="100000000001",id="100000000002"
    pub fn insert_x_cos_grant_full_control(
        &mut self,
        x_cos_grant_full_control: String,
    ) -> &mut Self {
        self.headers.insert(
            "x-cos-grant-full-control".to_string(),
            x_cos_grant_full_control,
        );
        self
    }

    /// 定义存储桶的访问控制列表（ACL）属性。枚举值请参见 ACL 概述 文档中存储桶的预设 ACL 部分，
    /// 如 private，public-read 等，默认为 private
    pub fn insert_bucket_x_cos_acl(&mut self, x_cos_acl: BucketAcl) -> &mut Self {
        let v;
        match x_cos_acl {
            BucketAcl::AuthenticatedRead => v = "authenticated-read",
            BucketAcl::PRIVATE => v = "private",
            BucketAcl::PublicRead => v = "publish-read",
            BucketAcl::PublicReadWrite => v = "public-read-write",
        }
        self.headers.insert("x-cos-acl".to_string(), v.to_string());
        self
    }

    /// 赋予被授权者写入存储桶的权限，格式为 id="\[OwnerUin\]"，
    /// 如 id="100000000001"，可使用半角逗号（,）分隔多组被授权者，如 id="100000000001",id="100000000002"
    pub fn insert_bucket_x_cos_grant_write(&mut self, x_cos_grant_write: String) -> &mut Self {
        self.headers
            .insert("x-cos-grant-write".to_string(), x_cos_grant_write);
        self
    }
}

#[cfg(test)]
mod test {

    use crate::acl;

    #[test]
    fn test_acl() {
        let mut acl_header = acl::AclHeader::new();
        acl_header
            .insert_bucket_x_cos_acl(acl::BucketAcl::PublicRead)
            .insert_x_cos_grant_read("x-cos-grant-read".to_string())
            .insert_x_cos_grant_read_acp("x_cos_grant_read_acp".to_string())
            .insert_x_cos_grant_write_acp("x_cos_grant_write_acp".to_string())
            .insert_bucket_x_cos_grant_write("x_cos_grant_write".to_string());

        assert_eq!(acl_header.headers["x-cos-acl"], "publish-read".to_string());
        assert_eq!(
            acl_header.headers["x-cos-grant-read"],
            "x-cos-grant-read".to_string()
        );
        assert_eq!(
            acl_header.headers["x-cos-grant-read-acp"],
            "x_cos_grant_read_acp".to_string()
        );
        assert_eq!(
            acl_header.headers["x-cos-grant-write-acp"],
            "x_cos_grant_write_acp".to_string()
        );
        assert_eq!(
            acl_header.headers["x-cos-grant-write"],
            "x_cos_grant_write".to_string()
        );
    }
}
