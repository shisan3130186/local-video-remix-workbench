# FFmpeg 打包资源说明

这个目录用于 Windows 用户测试包内置 FFmpeg / FFprobe。

构建用户测试包前，需要运行项目脚本：

```powershell
.\scripts\prepare-bundled-ffmpeg.ps1
```

脚本会检查并准备：

- `ffmpeg.exe`
- `ffprobe.exe`

对应的许可证、来源说明和版本清单分别由`src-tauri/resources/legal`与本目录`manifest.json`维护。

当前 Git 不包含这两个大型第三方二进制文件，只提交版本清单、校验值和准备脚本。Rust 会依次查找：

1. `FFMPEG_PATH` / `FFPROBE_PATH` 指定的位置。
2. 程序旁边的 `resources/ffmpeg` 目录。
3. 程序旁边的 `ffmpeg` 目录或程序同级目录。
4. 开发项目中的本目录。
5. 系统 PATH。

Tauri打包时会把两项程序放进安装目录的`ffmpeg`文件夹。普通用户安装后不需要配置PATH或环境变量。
