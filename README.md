# Windsurf Account Manager

一个基于 Tauri + Vue 3 + TypeScript 开发的 Windsurf 多账号管理桌面应用，用于管理多个 Windsurf 账号并提供积分重置、账单查询等功能。

## 功能特性

### 账号管理
- ✅ 添加、编辑、删除账号
- ✅ 账号分组和标签管理
- ✅ 账号状态实时显示
- ✅ 密码和Token安全加密存储（AES-256-GCM）
- ✅ 批量账号操作

### API功能
- ✅ 积分重置功能：通过座位数更新实现积分重置（仅执行一次座位更新）
  - 批量重置支持并发执行（最多5个并发）
- ✅ Token自动刷新
- ✅ 账单信息查询
- ✅ Firebase认证集成
- ✅ 批量积分重置
- ✅ 座位数智能切换（3/4/5轮换）
- ✅ 全局刷新：右上角刷新按钮可同时更新所有账号的详细信息（配额、套餐、订阅等）

### 数据安全
- ✅ 使用系统密钥链存储加密密钥（Windows Credential Manager）
- ✅ 所有敏感信息均加密存储
- ✅ 本地JSON文件存储
- ✅ 操作日志记录

## 技术栈

### 前端
- **框架**: Vue 3 + TypeScript
- **UI组件**: Element Plus
- **状态管理**: Pinia
- **构建工具**: Vite
- **样式**: CSS3 + Element Plus主题

### 后端
- **框架**: Tauri 2.x
- **语言**: Rust
- **加密**: AES-256-GCM
- **密钥管理**: Windows Credential Manager / Keyring
- **网络请求**: Reqwest
- **异步运行时**: Tokio

## 安装和运行

### 前提条件
- Node.js 16+
- Rust 1.70+
- Windows 10/11（目前仅支持Windows）

### 开发环境

```bash
# 克隆项目
git clone [项目地址]
cd windsurf-account-manager

# 安装依赖
npm install

# 开发模式运行
npm run tauri dev
```

### 构建发布版

```bash
# 构建Windows安装包
npm run tauri build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/`

## 使用说明

### 1. 首次使用

1. 启动应用后，点击"添加账号"按钮
2. 输入Windsurf账号信息：
   - 邮箱：你的Windsurf账号邮箱
   - 密码：账号密码
   - 备注名称：便于识别的名称
   - 分组（可选）：账号分组
   - 标签（可选）：自定义标签
3. 点击确定保存账号

### 2. 积分重置

1. 在账号卡片上点击"积分重置"按钮
2. 应用会自动：
   - 登录获取Token（如需要）
   - 执行一次座位数更新（在3/4/5之间自动切换）
   - 座位更新成功即完成积分重置
3. 操作结果会通过通知提示
4. 可在设置中开启"显示详细结果"查看座位更新的具体信息

### 3. 批量操作

1. 勾选多个账号卡片
2. 点击顶部的"批量重置积分"或"批量删除"按钮
3. 确认操作后批量执行

### 4. 账号分组

1. 点击侧边栏的"分组"菜单
2. 选择已有分组或添加新分组
3. 在添加/编辑账号时选择分组

### 5. 查看日志

1. 点击侧边栏的"操作日志"
2. 查看所有操作记录
3. 可以清空或导出日志

## 数据存储

应用数据存储在本地：
- **Windows**: `%APPDATA%\com.chao.windsurf-account-manager\accounts.json`

数据结构包括：
- 账号信息（加密的密码和Token）
- 分组列表
- 系统设置
- 操作日志

## 安全说明

1. **密码安全**：所有密码使用AES-256-GCM加密
2. **密钥管理**：加密密钥存储在系统密钥链中
3. **Token刷新**：Token过期前5分钟自动刷新
4. **本地存储**：所有数据仅存储在本地，不会上传到任何服务器

## 注意事项

1. 请妥善保管你的账号信息
2. 定期备份 `accounts.json` 文件
3. 批量操作时注意API限流
4. 建议使用分组功能管理多个账号

## 开发说明

### 项目结构

```
windsurf-account-manager/
├── src/                    # Vue前端源码
│   ├── views/             # 页面组件
│   ├── components/        # 可复用组件
│   ├── store/            # Pinia状态管理
│   ├── api/              # API封装
│   ├── types/            # TypeScript类型定义
│   └── utils/            # 工具函数
├── src-tauri/            # Rust后端源码
│   ├── src/
│   │   ├── models/       # 数据模型
│   │   ├── repository/   # 数据访问层
│   │   ├── services/     # 业务逻辑层
│   │   ├── commands/     # Tauri命令层
│   │   └── utils/        # 工具函数
│   └── Cargo.toml        # Rust依赖配置
├── package.json          # Node依赖配置
└── vite.config.ts        # Vite配置
```

### API集成

应用集成了以下Windsurf API：
- Firebase认证API
- 座位管理API（UpdateSeats）
- 账单查询API（GetTeamBilling）
- 计划更新API（UpdatePlan）

#### UpdateSeats API 响应数据说明

座位更新API返回的解析后JSON包含：
- `success`: 操作是否成功
- `total_seats`: 总座位数（对应int_4字段）
- `used_seats`: 已使用座位数（对应int_5字段）
- `seat_usage`: 座位使用情况（如："1 / 5"）
- `seat_usage_percentage`: 座位使用率百分比（计算得出）
- `price_per_seat`: 每座位月费（对应float_3字段，单位：美元）
- `total_monthly_price`: 总月费（对应float_6字段，单位：美元）
- `billing_start_time`: 当前计费周期开始时间（对应subMesssage_7.int_1时间戳）
- `next_billing_time`: 下次计费时间（对应subMesssage_8.int_1时间戳）
- `billing_start_timestamp`: 计费开始时间戳（Unix时间戳）
- `next_billing_timestamp`: 下次计费时间戳（Unix时间戳）

**Protobuf字段映射**：
- `float_3`: 每座位月费价格（如：60 = $60/月/座位）
- `int_4`: 总座位数（如：5）
- `int_5`: 已使用座位数（如：1）
- `float_6`: 总月费（如：300 = $300/月）
- `subMesssage_7.int_1`: 当前计费周期开始时间戳
- `subMesssage_8.int_1`: 下次计费时间戳

#### GetTeamBilling API 响应数据说明

账单查询API返回的解析后JSON包含：
- `plan_name`: 套餐名称
- `base_quota`: 套餐基础额度（对应int_8字段）
- `extra_credits`: 额外积分（对应int_4字段，可选）
- `total_quota`: 总额度（base_quota + extra_credits）
- `used_quota`: 已使用额度（对应int_6字段，可选，默认0）
- `cache_limit`: 缓存限额（对应int_9字段）
- `payment_method`: 支付方式信息
- `next_billing_date`: 下次扣费日期
- `invoice_url`: 发票链接
- `monthly_price`: 月费价格

**Protobuf字段映射**：
- `int_4`: 额外积分（赠送或购买的额外额度，可能不存在）
- `int_6`: 使用积分（已使用额度，可能不存在）
- `int_8`: 套餐额度（基础套餐额度）
- `int_9`: 套餐缓存限额（超过此限额将无法使用）

**注意**：当缓存使用率达到缓存限额时，即使总额度还有剩余也将无法继续使用。

## 常见问题

### Q: 为什么积分重置失败？
A: 请检查：
1. 账号密码是否正确
2. 网络连接是否正常
3. Token是否过期（尝试重新登录）

### Q: 如何备份账号数据？
A: 复制 `%APPDATA%\com.chao.windsurf-account-manager\accounts.json` 文件到安全位置

### Q: 忘记了账号密码怎么办？
A: 应用中存储的密码是加密的，无法查看明文。需要重新添加账号。

## 贡献指南

欢迎提交Issue和Pull Request！

## 许可证

MIT License

## 免责声明

本工具仅供学习和个人使用，请遵守Windsurf服务条款。作者不对因使用本工具产生的任何问题负责。
