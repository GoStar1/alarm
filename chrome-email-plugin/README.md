# Chrome Email Sender Plugin

## 功能
点击浏览器插件按钮，发送 GET 请求到 http://localhost:8888/send-email，触发邮件发送。

## 安装步骤

### 1. 添加图标
在 `images` 目录中添加以下图标文件：
- `icon16.png` (16x16 pixels)
- `icon48.png` (48x48 pixels)
- `icon128.png` (128x128 pixels)

您可以使用任何图片编辑工具创建这些图标，或者从网上下载免费的图标。

### 2. 加载插件到 Chrome
1. 打开 Chrome 浏览器，访问 `chrome://extensions/`
2. 开启右上角的 "开发者模式"
3. 点击 "加载已解压的扩展程序"
4. 选择 `chrome-email-plugin` 目录
5. 插件将被加载到 Chrome 中

### 3. 配置本地服务器
确保您的本地服务器在 http://localhost:8888 上运行，并且有一个 `/send-email` 端点来处理邮件发送请求。

## 使用方法
1. 点击 Chrome 工具栏中的插件图标
2. 在弹出的窗口中点击 "Send Test Email" 按钮
3. 插件会发送 GET 请求到本地服务器，并显示操作状态

## 注意事项
- 确保本地服务器正在运行
- 确保服务器端口与插件中配置的端口一致 (8888)
- 确保服务器允许来自 Chrome 插件的跨域请求

## 调试
- 在 Chrome 中右键点击插件图标，选择 "检查弹出窗口"
- 在控制台中查看详细的日志信息
