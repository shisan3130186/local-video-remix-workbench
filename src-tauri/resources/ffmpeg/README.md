# FFmpeg 打包资源说明

这个目录是未来 Windows 安装包内置 FFmpeg / FFprobe 的预留位置。

发给普通测试用户前，需要准备与许可证策略一致的：

- `ffmpeg.exe`
- `ffprobe.exe`
- 对应的许可证和来源说明

当前 Git 不包含这两个大型第三方二进制文件。Rust 会依次查找：

1. `FFMPEG_PATH` / `FFPROBE_PATH` 指定的位置。
2. 程序旁边的 `resources/ffmpeg` 目录。
3. 程序旁边的 `ffmpeg` 目录或程序同级目录。
4. 开发项目中的本目录。
5. 系统 PATH。

只有把合法来源的二进制加入安装包资源后，干净电脑才可以不单独安装 FFmpeg。
