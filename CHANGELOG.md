# Changelog

## [0.1.13] - 2024-10-01

### Changed

- 优化大文件上传，每个线程会尝试 10 次

### Fixed

- 修复`put_object`方法报`不是内存对象`的报错

## [0.1.12] - 2024-08-23

### Changed

- 优化文件的下载，支持边下载边保存

- 移除`async-trait`依赖，不再基于 trait 来实现方法

### Added

- 新增 object `get_object_binary_range`方法，分块获取文件数据

## [0.1.11] - 2024-08-17

### Fixed

- Fix download file when object not exist

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
