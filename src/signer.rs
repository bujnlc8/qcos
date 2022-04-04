//! 接口签名
use chrono::Utc;
use ring::hmac;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::str;
use urlencoding::{decode, encode};

/// [文档](https://cloud.tencent.com/document/product/436/7778)
pub struct Signer<'a> {
    method: &'a str,
    url_path: &'a str,
    headers: Option<&'a HashMap<String, String>>,
    query: Option<&'a HashMap<String, String>>,
}

impl<'a> Signer<'a> {
    pub fn new(
        method: &'a str,
        url_path: &'a str,
        headers: Option<&'a HashMap<String, String>>,
        query: Option<&'a HashMap<String, String>>,
    ) -> Self {
        Self {
            method,
            url_path,
            headers,
            query,
        }
    }

    fn get_key_time(&self, valid_seconds: u32) -> String {
        let start = Utc::now().timestamp();
        let end = start + valid_seconds as i64;
        format!("{};{}", start, end)
    }

    fn get_sign_key(&self, key_time: &str, secret_key: &str) -> String {
        let key = hmac::Key::new(
            hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
            secret_key.to_string().as_bytes(),
        );
        let signature = hmac::sign(&key, key_time.as_bytes());
        let s: Vec<String> = signature
            .as_ref()
            .into_iter()
            .map(|x| format!("{:02x?}", x))
            .collect();
        s.join("")
    }

    fn encode_data(&self, data: &HashMap<String, String>) -> HashMap<String, String> {
        let mut res = HashMap::new();
        for (k, v) in data.iter() {
            res.insert(encode(k).to_string().to_lowercase(), encode(v).to_string());
        }
        res
    }

    fn get_url_param_list(&self) -> String {
        if let Some(ref query) = self.query {
            let mut keys: Vec<String> = Vec::new();
            let encoded_data = self.encode_data(&query);
            for k in encoded_data.keys() {
                keys.push(k.to_string());
            }
            keys.sort();
            return keys.join(";");
        }
        "".to_string()
    }

    fn get_http_parameters(&self) -> String {
        if let Some(ref query) = self.query {
            let mut keys: Vec<String> = Vec::new();
            let encoded_data = self.encode_data(&query);
            for k in encoded_data.keys() {
                keys.push(k.to_string());
            }
            keys.sort();
            let mut res: Vec<String> = Vec::new();
            for key in keys {
                let v = encoded_data.get(&key).unwrap();
                res.push(vec![key, v.to_string()].join("="));
            }
            return res.join("&");
        }
        "".to_string()
    }

    fn get_header_list(&self) -> String {
        if let Some(ref headers) = self.headers {
            let mut keys: Vec<String> = Vec::new();
            let encoded_data = self.encode_data(&headers);
            for k in encoded_data.keys() {
                keys.push(k.to_string());
            }
            keys.sort();
            return keys.join(";");
        }
        "".to_string()
    }

    fn get_heades(&self) -> String {
        if let Some(ref headers) = self.headers {
            let mut keys: Vec<String> = Vec::new();
            let encoded_data = self.encode_data(&headers);
            for k in encoded_data.keys() {
                keys.push(k.to_string());
            }
            keys.sort();
            let mut res: Vec<String> = Vec::new();
            for key in keys {
                let v = encoded_data.get(&key).unwrap();
                res.push(vec![key, v.to_string()].join("="));
            }
            return res.join("&");
        }
        "".to_string()
    }

    fn get_http_string(&self) -> String {
        let s = vec![
            self.method.to_string(),
            decode(self.url_path).unwrap().to_string(),
            self.get_http_parameters(),
            self.get_heades(),
        ];
        s.join("\n") + "\n"
    }

    fn get_string_to_sign(&self, key_time: &'a str) -> String {
        let mut s = vec!["sha1".to_string(), key_time.to_string()];
        let http_string = self.get_http_string();
        let mut hasher = Sha1::new();
        hasher.update(&http_string);
        let result = hasher.finalize();
        let digest: Vec<String> = result
            .as_slice()
            .into_iter()
            .map(|x| format!("{:02x?}", x))
            .collect();
        s.push(digest.join(""));
        s.join("\n") + "\n"
    }

    pub fn get_signature(&self, secret_key: &str, secret_id: &str, valid_seconds: u32) -> String {
        let key_time = self.get_key_time(valid_seconds);
        let string_to_sign = self.get_string_to_sign(&key_time);
        let sign_key = self.get_sign_key(&key_time, secret_key);
        let signature = self.get_sign_key(&string_to_sign, &sign_key);
        let header_list = self.get_header_list();
        let param_list = self.get_url_param_list();
        format!("q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}&q-header-list={}&q-url-param-list={}&q-signature={}", secret_id, key_time, key_time, header_list, param_list, signature)
    }
}

#[cfg(test)]
mod test {
    use crate::signer::Signer;
    use std::collections::HashMap;

    #[test]
    fn test_get_key_time() {
        let signer = Signer::new("", "", None, None);
        println!("{}", signer.get_key_time(100));
    }

    #[test]
    fn test_get_url_param_list() {
        let mut query = HashMap::new();
        query.insert("a".to_string(), "a ".to_string());
        query.insert("B".to_string(), " b".to_string());
        let signer = Signer::new("", "", None, Some(&query));
        let s = signer.get_url_param_list();
        assert_eq!(s, "a;b");
        let s = signer.get_http_parameters();
        assert_eq!(s, "a=a%20&b=%20b");
    }

    #[test]
    fn test_get_http_string() {
        let mut query = HashMap::new();
        query.insert("a".to_string(), "a ".to_string());
        query.insert("B".to_string(), " b".to_string());
        let mut headers = HashMap::new();
        headers.insert("h".to_string(), "h".to_string());
        headers.insert("user-agent".to_string(), "test".to_string());
        let signer = Signer::new("get", "/path", Some(&headers), Some(&query));
        assert_eq!(
            signer.get_http_string(),
            "get\n/path\na=a%20&b=%20b\nh=h&user-agent=test\n"
        );
        assert_eq!(
            signer.get_string_to_sign("1648999396;1648999496"),
            "sha1\n1648999396;1648999496\n963bfe30ee40d402ee00506981bab650e72134f6\n"
        );
    }

    #[test]
    fn test_get_signature() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/plain".to_string());
        headers.insert("Content-Length".to_string(), "13".to_string());
        headers.insert(
            "Host".to_string(),
            "examplebucket-1250000000.cos.ap-beijing.myqcloud.com".to_string(),
        );
        headers.insert(
            "Content-MD5".to_string(),
            "mQ/fVh815F3k6TAUm8m0eg==".to_string(),
        );
        headers.insert("x-cos-acl".to_string(), "private".to_string());
        headers.insert(
            "x-cos-grant-read".to_string(),
            "uin=\"100000000011\"".to_string(),
        );
        headers.insert(
            "Date".to_string(),
            "Thu, 16 May 2019 06:45:51 GMT".to_string(),
        );
        let signer = Signer::new(
            "put",
            "/exampleobject(%E8%85%BE%E8%AE%AF%E4%BA%91)",
            Some(&headers),
            None,
        );
        let key_time = "1557989151;1557996351";
        assert_eq!(
            signer.get_sign_key(key_time, "BQYIM75p8x0iWVFSIgqEKwFprpRSVHlz"),
            "eb2519b498b02ac213cb1f3d1a3d27a3b3c9bc5f"
        );

        assert_eq!(signer.get_url_param_list(), "");
        assert_eq!(signer.get_http_parameters(), "");
        assert_eq!(
            signer.get_header_list(),
            "content-length;content-md5;content-type;date;host;x-cos-acl;x-cos-grant-read"
        );

        assert_eq!(signer.get_heades(), "content-length=13&content-md5=mQ%2FfVh815F3k6TAUm8m0eg%3D%3D&content-type=text%2Fplain&date=Thu%2C%2016%20May%202019%2006%3A45%3A51%20GMT&host=examplebucket-1250000000.cos.ap-beijing.myqcloud.com&x-cos-acl=private&x-cos-grant-read=uin%3D%22100000000011%22");

        assert_eq!(signer.get_http_string(), "put\n/exampleobject(腾讯云)\n\ncontent-length=13&content-md5=mQ%2FfVh815F3k6TAUm8m0eg%3D%3D&content-type=text%2Fplain&date=Thu%2C%2016%20May%202019%2006%3A45%3A51%20GMT&host=examplebucket-1250000000.cos.ap-beijing.myqcloud.com&x-cos-acl=private&x-cos-grant-read=uin%3D%22100000000011%22\n");

        assert_eq!(
            signer.get_string_to_sign(key_time),
            "sha1\n1557989151;1557996351\n8b2751e77f43a0995d6e9eb9477f4b685cca4172\n"
        );
    }
}
