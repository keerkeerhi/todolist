# 先把 src-tauri/tauri.conf.json 的 version 改成新版本
git add .
git commit -m "release: v0.1.2"
git tag v0.1.2
git push origin main
git push origin v0.1.2


所以执行：git push origin v0.1.2

后，GitHub Actions 会自动：
1. 安装依赖；
2. 构建 Windows NSIS 安装包；
3. 使用签名密钥生成更新签名；
4. 创建 GitHub Release；
5. 上传 .exe、.zip、.sig 和 latest.json。
前提是：
- workflow 文件已经提交并推送到远程仓库；
- GitHub Secrets 名称正确；
- src-tauri/tauri.conf.json 的版本与发布 Tag 对应；
- Actions 构建没有失败。
构建状态可以在 GitHub 仓库的 Actions 页面查看。成功后，在 Releases 页面可以下载 Windows 安装包。