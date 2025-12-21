# WU-Blocker

![Python](https://img.shields.io/badge/Python-3.x-blue.svg)
![Platform](https://img.shields.io/badge/Platform-Windows-win.svg)
![License](https://img.shields.io/badge/License-MIT-green.svg)

这是一个轻量级的 Windows 系统工具，旨在帮助用户一键**暂停**或**恢复** Windows 自动更新。

将暂停更新的时间延长至 **`9999年12月31日`**（意义上的永久暂停）

## 📸 截图
<a href="#"><img alt="image" src="/IMG/1.png" /></a>

## ✅ 优势
对于传统的工具来说，我们只针对**Windows自动更新**，不影响`Windows Update`服务，对于需要使用微软商店的用户十分友好。

## 🛠️ 原理

修改以下注册表路径的键值：
`HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings`

**暂停更新时写入的键值：**
* `PauseFeatureUpdatesStartTime` / `EndTime` 
* `PauseQualityUpdatesStartTime` / `EndTime`
* `FlightSettingsMaxPauseDays` 

**恢复更新时：**
删除上述键值，让 Windows 恢复默认策略。

## ⬇️ 下载使用

前往 [Releases](https://github.com/NeetheCheeBao/WU-Blocker/releases) 页面下载。

1. 右键以**管理员身份运行**（程序会自动请求提权）。
2. 点击“永久暂停更新”即可。

## ⚖️ 许可证

本项目基于 [MIT License](LICENSE) 开源。
