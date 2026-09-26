<div align="center">

# Windows Update Blocker

[![Version](https://img.shields.io/badge/version-1.0.1-blue.svg)](#)
[![Python](https://img.shields.io/badge/Python-3.8%2B-blue.svg)](#)
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6.svg?logo=windows&logoColor=white)](#)
[![GUI](https://img.shields.io/badge/GUI-Tkinter-orange.svg)](#)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

这是一个轻量级的 Windows 系统工具，旨在帮助用户一键**暂停**或**恢复** Windows 自动更新。

将暂停更新的时间延长至 **9999年12月31日**（意义上的永久暂停）

</div>

## 📸 截图展示
![IMG](/screenshot/demo1.png)
![IMG](/screenshot/demo2.png)

## ✅ 优势
对于传统的工具来说，我们只针对**Windows自动更新**，不影响`Windows Update`服务，对于需要使用微软产品的用户十分友好。

## 🛠️ 原理

修改以下注册表路径的键值：
`HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings`

**暂停更新时写入的键值：**
* `FlightSettingsMaxPauseDays` 允许暂停更新的最大天数
* `PauseFeatureUpdatesEndTime` 功能更新暂停开始时间
* `PauseFeatureUpdatesStartTime` 功能更新暂停结束时间
* `PauseQualityUpdatesEndTime` 质量更新暂停开始时间（日常补丁、安全更新）
* `PauseQualityUpdatesStartTime` 质量更新暂停结束时间
* `PauseUpdatesExpiryTime` 所有更新暂停开始时间
* `PauseUpdatesStartTime` 暂停更新到期时间

**恢复更新：**
删除上述键值，即可让 Windows 恢复默认策略。

## 🛠️ 本地编译

```bash
.\build.bat
```

## ⬇️ 下载使用

前往 [Releases](https://github.com/NeetheCheeBao/WU-Blocker/releases) 页面下载。

1. 右键以**管理员身份运行**（程序会自动请求提权）。
2. 点击“永久暂停更新”即可。

## ⚖️ 许可证

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。
