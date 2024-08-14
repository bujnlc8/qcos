# Changelog

## [0.1.10] - 2024-08-14

### Changed

- 将文件上传路径字段`file_path`的类型从`&str`改成`&PathBuf`

## [0.1.9] - 2024-08-08

### Changed

- `Response` 的 `result` 字段的类型从`bytes.Byte`替换成`Vec<u8>`

- 通过参数传递请求头的时候，类型变更成`HeaderMap`, 而不是之前的`HashMap<String, String>`

### Added

- 下载文件支持显示进度条，需要开启`progress-bar`feature

- 下载文件支持多线程下载

- 新增查询文件大小的方法`get_object_size`

### Fixed

- Fix some docs
