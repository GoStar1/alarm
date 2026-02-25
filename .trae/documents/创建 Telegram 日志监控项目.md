# Telegram 日志监控项目

## 项目目标
创建一个 Rust 程序，监控 Telegram 日志并在特定条件下发送 Telegram 消息通知。

## 功能需求
1. 监控 `log stream --predicate 'process == "Telegram"' --level info` 日志
2. 当出现 "Telegram: (UserNotifications)" 时，发送通知："合约群里有新消息，请注意！！！"
3. 当 1 分钟内没出现 "Telegram: (Network)" 时，发送通知："本地telegram网络错误"
4. 使用 Telegram Bot API 发送通知

## 技术方案
1. **项目结构**：
   - 创建新的 `telegram-log-monitor` 目录
   - 使用 Rust 语言实现
   - 依赖：tokio、reqwest、subprocess、tracing

2. **核心功能**：
   - 使用 `subprocess` 执行 `log stream` 命令
   - 实时读取日志输出
   - 使用 `Arc<AtomicBool>` 跟踪网络活动状态
   - 使用 `tokio::time::interval` 实现 1 分钟检查
   - 使用 `reqwest` 发送 Telegram API 请求

3. **实现步骤**：
   - 初始化 Rust 项目
   - 配置依赖
   - 实现日志监控逻辑
   - 实现网络活动检查
   - 实现 Telegram 消息发送
   - 测试运行

4. **注意事项**：
   - `log stream` 命令需要管理员权限
   - 确保网络连接正常
   - 处理可能的错误情况
   - 提供详细的日志输出

## 预期结果
创建一个可运行的 Rust 程序，能够：
- 监控 Telegram 日志
- 在检测到新消息时发送通知
- 在网络活动异常时发送通知
- 提供清晰的日志输出
- 稳定运行