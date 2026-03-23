## rs\_elib

基于 Rust + Axum + SeaORM + SQLite 构建的轻量级个人电子图书馆管理系统，支持 EPUB、TXT 等格式电子书的解析和在线阅读。

## 启动项目

正式环境启动之前，请通过环境变量配置`JWT_SECRET`，以设置JWT密钥。请不要使用默认值，否则会导致安全风险！

```bash
# 开发模式
cargo run

# Release模式
cargo build --release
./target/release/rs_elib

# 嵌入文本方式加载HTML等资源
cargo build --release --features "embed_static"

```

如果使用`embed_static`特征，程序会将`../static/index.min.html`嵌入到可执行文件中，但部分css/js资源仍需要从网络引入以及本地static目录引入。

启动参数如下:

| 完整参数            | 参数类型 | 默认值                       | 参数说明                 |
| --------------- | ---- | ------------------------- | -------------------- |
| `--host`        | 字符串  | `127.0.0.1`               | 服务监听 IP 地址           |
| `--port`        | 整数   | `3000`                    | 服务监听端口号              |
| `--upload-dir`  | 字符串  | `./uploads`               | 文件上传存储目录             |
| `--log-level`   | 枚举   | `info`                    | 日志输出级别               |
| `--db_conn_str` | 字符串  | `sqlite:elib.db?mode=rwc` | 数据库连接字符串（seaorm支持类型） |

服务器默认运行在 `http://127.0.0.1:3000`

# Docker使用说明

本文件提供了使用Docker构建和运行rs\_elib应用的详细说明。

## 构建Docker镜像

### 基本构建

```bash
# 进入项目根目录
cd rs_elib

# 构建Docker镜像
docker build -t rs_elib .
```

### 构建时指定标签

```bash
docker build -t rs_elib:v1.0 .
```

## 运行Docker容器

### 基本运行

```bash
docker run -p 3000:3000 rs_elib
```

### 运行时设置环境变量

```bash
docker run -p 3000:3000 \
  -e JWT_SECRET="your_secure_secret" \
  rs_elib
```

### 运行时传递命令行参数

```bash
docker run -p 3000:3000 \
  rs_elib --host 0.0.0.0 --port 3000
```

### 同时设置环境变量和命令行参数

```bash
docker run -p 3000:3000 \
  -e JWT_SECRET="your_secure_secret" \
  rs_elib --host 0.0.0.0 --port 3000 --upload-dir /app/uploads
```

### 挂载上传目录

```bash
docker run -p 3000:3000 \
  -e JWT_SECRET="your_secure_secret" \
  -v /path/to/local/uploads:/app/uploads \
  rs_elib --upload-dir /app/uploads
```


### 示例：完整的生产环境配置

```bash
docker run -d \
  --name rs_elib \
  -p 80:3000 \
  -e JWT_SECRET="your_very_secure_secret_key" \
  -v /data/rs_elib/uploads:/app/uploads \
  -v /data/rs_elib/db:/app/db \
  rs_elib \
  --host 0.0.0.0 \
  --port 3000 \
  --upload-dir /app/uploads \
  --db-conn-str sqlite:///app/db/elib.db \
  --log-level warn
```

### 查看容器日志

```bash
docker logs rs_elib
```

### 进入容器

```bash
docker exec -it rs_elib bash
```

### 停止和删除容器

```bash
# 停止容器
docker stop rs_elib

# 删除容器
docker rm rs_elib
```

### 注意事项

1. 首次运行时，应用会自动创建SQLite数据库文件
2. 确保JWT\_SECRET在生产环境中设置为安全的随机字符串
3. 对于生产环境，建议使用外部数据库而不是SQLite
4. 上传目录应该挂载到宿主机，以防止容器重启后数据丢失



## 技术栈

- **后端框架**: Axum
- **数据库 ORM**: SeaORM
- **数据库**: SQLite
- **认证**: JWT Bearer Token
- **前端**: HTML + JavaScript + Tailwind CSS

## 文件存储

- 数据库: `./elib.db` (SQLite)
- 上传文件: `./uploads/` 目录
- 图书资源: `./uploads/book_{id}_resources/` 目录
- 静态文件: `./static/` 目录

## 用户权限

### 角色

本项目定义了4种用户角色，权限层级如下：

- **User (普通用户)**：仅拥有基础阅读和个人账户管理权限，为系统基础参与者。
- **Contributor (贡献者)**：继承普通用户所有权限，新增内容上传、分组创建/管理权限，为系统内容建设者。
- **Admin (管理员)**：继承贡献者所有权限，新增全量用户、图书、分组及系统设置管理权限，为系统运营者。
- **Root (超级管理员)**：继承管理员所有权限，新增修改/删除Admin角色的权限，拥有系统最高控制权。**注册第一位用户即为root**。

### 权限层级关系

角色权限具有层级继承关系，高级角色包含低级角色的所有权限：

```
Root > Admin > Contributor > User
```

### 权限表

说明：✅ 表示拥有该权限，❌ 表示无该权限

| 权限分类       | 权限操作                  | User | Contributor | Admin | Root |
| :--------- | :-------------------- | :--: | :---------: | :---: | :--: |
| **基础账户权限** | 查看自己所属分组的图书           |   ✅  |      ✅      |   ✅   |   ✅  |
| **基础账户权限** | 阅读自己有权限的图书            |   ✅  |      ✅      |   ✅   |   ✅  |
| **基础账户权限** | 修改自己的密码               |   ✅  |      ✅      |   ✅   |   ✅  |
| **基础账户权限** | 使用邀请码加入分组             |   ✅  |      ✅      |   ✅   |   ✅  |
| **内容管理权限** | 上传图书                  |   ❌  |      ✅      |   ✅   |   ✅  |
| **内容管理权限** | 管理自己上传的图书（编辑、删除）      |   ❌  |      ✅      |   ✅   |   ✅  |
| **内容管理权限** | 管理所有图书                |   ❌  |      ❌      |   ✅   |   ✅  |
| **分组管理权限** | 创建分组                  |   ❌  |      ✅      |   ✅   |   ✅  |
| **分组管理权限** | 管理自己创建的分组             |   ❌  |      ✅      |   ✅   |   ✅  |
| **分组管理权限** | 管理所有分组                |   ❌  |      ❌      |   ✅   |   ✅  |
| **用户管理权限** | 管理所有用户（创建、编辑角色、禁用、删除） |   ❌  |      ❌      |   ✅   |   ✅  |
| **用户管理权限** | 修改 Admin 角色           |   ❌  |      ❌      |   ❌   |   ✅  |
| **用户管理权限** | 删除 Admin 用户           |   ❌  |      ❌      |   ❌   |   ✅  |
| **系统管理权限** | 管理系统设置                |   ❌  |      ❌      |   ✅   |   ✅  |
| **系统最高权限** | 不能禁用 Root 用户          |   -  |      -      |   -   |   ✅  |
| **系统最高权限** | 不能删除 Root 用户          |   -  |      -      |   -   |   ✅  |
| **系统最高权限** | 不能禁用自己                |   -  |      -      |   -   |   ✅  |
| **系统最高权限** | 不能删除自己                |   -  |      -      |   -   |   ✅  |

## AppState

`AppState`定义于`./src/state.rs`。对于Axum框架来讲需要拥有一个全局共享状态，用于存储数据库连接、上传目录、格式解析器注册器等。Axum的`State<T>`默认要求 T: Clone + Send + Sync，原因是Axum handler 可能在多线程上被并发调用，每次请求都需要一个状态副本。

## 格式解析器

本项目于`./src/format/mod.rs`中定义`FormatParser`的trait，用于解析不同格式的电子书。

### FormatParser Trait

```rs
// ./src/format/mod.rs
pub trait FormatParser: Send + Sync{
    fn new(path: &str) -> Self
        where Self: Sized;

    /// 多次调用直到返回`FormatParserError::EndOfChapters`
    fn parse_chapters<'a>(
        &'a mut self,
        book_id: i32,
    ) -> BoxFuture<'a, Result<ChapterParsed>>;
}
```

其中`new`方法用于传递解析文件路径。

关于`parse_chapters`方法，会被系统多次调用，每一次调用都需要返回使用`Result<ChapterParsed>`包装的结构体，根据最后一次调用返回`FormatParserError::EndOfChapters`异常标志着解析结束。

关于`ChapterParsed`结构体和`Result<T>`，定义如下：

```rs
// ./src/format/mod.rs
pub type Result<T> = std::result::Result<T, FormatParserError>;

pub struct ChapterParsed {
    pub book_id: i32,             // 对应图书ID，会以parse_chapters方法中的参数传递
    pub chapter_number: i32,     // 章节序号
    pub title: String,          // 章节标题
    pub content: String,       // 章节内容
}
```

对于解析内容，默认的格式解析器输出内容都为HTML。

### FormatParserRegister - 解析器注册器

`FormatParserRegister` 提供了一个灵活的解析器注册和管理机制，方便第三方开发者扩展支持新的文件格式。

#### 解析器工厂模式

系统使用工厂模式来创建解析器实例，这样每个文件都可以获得一个全新的解析器实例。主要组件包括：

```rs
// 解析器工厂 trait
pub trait FormatParserFactory: Send + Sync {
    fn create(&self, path: &str) -> Arc<Mutex<dyn FormatParser + Send + Sync>>;
}

// 泛型解析器工厂
pub struct GenericParserFactory<T> {
    _marker: std::marker::PhantomData<T>,
}
```

#### 注册解析器

你可以在 `./src/main.rs` 中使用 `FormatParserRegister` 来注册自定义解析器：

```rs
// 注册解析器
parser_register.register::<MyCustomParser>("custom");
```

#### 获取解析器

系统会自动根据文件格式调用 `get()` 方法获取对应的解析器：

```rs
// 根据格式和文件路径获取解析器
let parser = state.parser_register.get(&file_type, &file_path);
```

### 第三方开发者扩展指南

要添加对新格式的支持，请按照以下步骤：

#### 1. 实现 FormatParser Trait

```rust
// src/format/custom.rs
use super::{ChapterParsed, FormatParser, FormatParserError, Result};
use futures::future::BoxFuture;

pub struct CustomParser {
    path: Option<String>,
    // 其他字段...
}

impl CustomParser {
    pub fn new(path: &str) -> Self {
        Self {
            path: Some(path.to_string()),
            // 初始化其他字段...
        }
    }
}

impl FormatParser for CustomParser {
    fn new(path: &str) -> Self {
        Self::new(path)
    }

    fn parse_chapters<'a>(
        &'a mut self,
        book_id: i32,
    ) -> BoxFuture<'a, Result<ChapterParsed>> {
        Box::pin(async move {
            // 实现解析逻辑
            // 返回 Ok(ChapterParsed) 或 Err(FormatParserError::EndOfChapters)
        })
    }
}
```

#### 2. 在 mod.rs 中导出

```rust
// src/format/mod.rs
pub mod custom;
```

#### 3. 在 main.rs 中注册

```rust
// src/main.rs
let mut parser_register = format::FormatParserRegister::new();
parser_register.register::<format::epub::EpubParser>("epub");
parser_register.register::<format::txt::TxtParser>("txt");
parser_register.register::<format::markdown::MarkdownParser>("markdown");
// 添加你的自定义解析器
parser_register.register::<format::custom::CustomParser>("custom");
```

完成这些步骤后，系统就会自动支持 .custom 格式的文件了！

### 默认EPUB解析器

EPUB 本质上是一个**ZIP 容器格式**，内部通过多个 XML 文件描述电子书结构。
在解析 EPUB 时，需要按照固定的**路径映射关系**逐级解析。

整体解析流程如下：

```
EPUB(zip)
   ↓
解析 container.xml
   ↓
定位 content.opf
   ↓
解析 manifest
   ↓
获取所有资源路径
   ↓
解压或缓存资源
   ↓
生成 API 访问路径
   ↓
重写 HTML 中的资源 URL
```

EPUB结构如下：

```
EPUB (zip)
 ├─ META-INF/container.xml
 │
 └─ OEBPS / OPS / EPUB
      ├─ content.opf
      │
      ├─ toc.ncx / nav.xhtml
      │
      └─ chapters (*.xhtml / *.html)
```

#### OPF路径

EPUB文件入口是

```
META-INF/container.xml
```

该问文件用于指定OPF文件的位置，如

```xml
<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0"
 xmlns="urn:oasis:names:tc:opendocument:xmlns:container">

  <rootfiles>
    <rootfile
      full-path="OEBPS/content.opf"
      media-type="application/oebps-package+xml"/>
  </rootfiles>

</container>
```

解析步骤：

- 打开EPUB文件，定位到META-INF/container.xml
- 读取container.xml，解析rootfiles节点
- 得到OPF文件路径OEBPS/content.opf

#### OPF文件结构

OPF文件是EPUB的**核心文件**，用于描述电子书的元数据、目录结构、资源列表等。

它包含如下几个部分：

| 部分       | 作用     |
| -------- | ------ |
| metadata | 书籍元数据  |
| manifest | 所有资源文件 |
| spine    | 阅读顺序   |

如下示例:

```xml
<package version="3.0">

  <metadata>
      <dc:title>Example Book</dc:title>
      <dc:creator>Author</dc:creator>
  </metadata>

  <manifest>

      <item id="nav"
            href="nav.xhtml"
            media-type="application/xhtml+xml"
            properties="nav"/>

      <item id="chapter1"
            href="chapter1.xhtml"
            media-type="application/xhtml+xml"/>

  </manifest>

  <spine>
      <itemref idref="chapter1"/>
  </spine>

</package>
```

#### Manifest文件映射路径

manifest定义了资源id到资源路径的映射关系。

例如

```
id -> href
```

示例：

```
nav -> nav.xhtml
chapter1  → chapter1.xhtml
chapter2  → chapter2.xhtml
```

其中href是相对于OPF的文件路径，例如

```
OPF: OEBPS/content.opf
href: chapter1.xhtml
```

真实路径为

```
OEBPS/chapter1.xhtml
```

在本项目解析的时候需要将EPUB文件的资源映射到API路径上，解析时候会生成资源映射表

```
EPUB资源路径          → API路径

chapter1.xhtml        → /api/books/{book_id}/resource/chapter1.xhtml
Images/cover.jpg      → /api/books/{book_id}/resource/Images/cover.jpg
Styles/style.css      → /api/books/{book_id}/resource/Styles/style.css
```

#### Spine阅读顺序

spine决定章节阅读顺序。

```xml
<spine>
    <itemref idref="chapter1"/>
    <itemref idref="chapter2"/>
</spine>
```

解析步骤：

- 读取 itemref/idref
- 在 manifest 查找对应 item
- 获取 href
- 构造真实路径

例如

```
chapter1 → chapter1.xhtml → OEBPS/chapter1.xhtml
chapter2 → chapter2.xhtml → OEBPS/chapter2.xhtml
```

最终得到章节顺序

```
OEBPS/chapter1.xhtml
OEBPS/chapter2.xhtml
```

#### 目录TOC

EPUB目录有两种格式

EPUB2 目录格式:

```
toc.ncx
```

Manifest 示例：

```xml
<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
```

EPUB3 目录格式:

```
nav.xhtml
```

Manifest实例：

```xml
<item
    id="nav"
    href="nav.xhtml"
    properties="nav"
    media-type="application/xhtml+xml"/>
```

#### 章节文件

章节通常是`*.xhtml`或`*.html`

解析时只需要：

- 提取`<body>`
- 保留HTML结构
- 或者转换为内部HTML格式

在解析章节文件时，所有资源路径都会被重写为API地址。

比如原始HTML

```html
<img src="Images/cover.jpg">
```

重写后为

```html
<img src="/api/books/1/resource/Images/cover.jpg">
```

这样浏览器加载章节时只需要

```
GET /api/books/1/resource/Images/cover.jpg
```

### 默认TXT解析器

`TxtParser`是本项目用于解析 TXT 电子书的解析器，实现了`FormatParser`接口。

其目标是：

- 支持超大 TXT 文件
- 自动拆分章节
- 按章节解析
- 输出 HTML 内容

因此，解析器不会一次性加载整个文件，而是采用**流式读取**。

TXT 解析流程如下：

```
TXT文件
   ↓
逐行读取
   ↓
匹配章节标题
   ↓
构建章节列表
   ↓
提取章节内容
   ↓
转换 HTML
   ↓
返回 ChapterParsed
```

#### 章节解析

解析器通过正则表达式自动识别章节标题，例如:

```
第1章
第一章
第十章
第1节
卷一
```

正则表达式规则如下：

```rust
Regex::new(
    r"^(第\s*[零一二三四五六七八九十百千万\d]+\s*[章节卷篇集部节]|卷\s*[零一二三四五六七八九十百千万\d]+\s*[：:]|[零一二三四五六七八九十百千万\d]+\s*[章节卷篇集部节]\s*[：:])"
).unwrap()
```

#### HTML转换

TXT 内容会转换为 HTML：

原始 TXT：

```txt
这是第一段

这是第二段
```

转换后为：

```html
<div>
这是第一段<br>
<br>
这是第二段
</div>
```

### 默认Markdown解析器

默认markdown解析使用`pulldown-cmark`库。详情请参考[pulldown-cmark](https://docs.rs/pulldown-cmark/0.9.0/pulldown_cmark/)

## API 接口文档

### 资源映射

资源通过API统一访问

```
GET /api/books/{book_id}/resource/{path}
```

示例

```
/api/books/1/resource/Images/cover.jpg
/api/books/1/resource/Styles/style.css
/api/books/1/resource/chapter1.xhtml
```

非常重要：对于前端请求的时候，需要携带`Bearer {Token}`。而对于浏览器`<img>`等标签，并不会自动添加认证信息。

### 公共接口 (无需认证)

#### POST /api/auth/register - 用户注册

**请求体:**

```json
{
  "username": "string",
  "password": "string"
}
```

**密码要求:** 8-16位，包含大小写字母、数字和特殊字符
**响应:**

- 201: 注册成功，返回用户信息
- 400: 密码不符合要求
- 403: 注册功能已禁用
- 409: 用户名已存在

#### POST /api/auth/login - 用户登录

**请求体:**

```json
{
  "username": "string",
  "password": "string"
}
```

**响应:**

```json
{
  "token": "jwt_token",
  "user": {
    "id": 1,
    "username": "string",
    "role": "string"
  }
}
```

- 200: 登录成功
- 401: 凭据无效
- 403: 账户已禁用

#### GET /health - 健康检查

**响应:** `OK`

***

### 认证接口 (需要 Bearer Token)

所有以下接口都需要在请求头中包含:

```


Authorization: Bearer <jwt_token>
```

#### GET /api/auth/me - 获取当前用户信息

**权限:** 所有用户
**响应:**

```json
{
  "id": 1,
  "username": "string",
  "role": "string"
}
```

***

### 图书接口

#### GET /api/books - 获取图书列表

**权限:**

- Admin/Root: 所有图书
- Contributor: 自己上传的图书 + 所属分组的图书
- User: 所属分组的图书
  **响应:** 图书数组

#### POST /api/books - 上传图书

**权限:** Admin/Root/Contributor
**请求格式:** multipart/form-data
**字段:**

- file: 图书文件 (必需)
- title: 书名 (必需)
- author: 作者 (可选)
- description: 描述 (可选)
- isbn: ISBN (可选)
- category: 分类 (可选)
  **响应:** 201 创建成功，返回图书信息
- 注: 系统会自动计算文件哈希值，如果相同哈希的图书已存在，直接返回已存在的图书

#### POST /api/books/search - 搜索图书

**权限:** 所有用户
**请求体:**

```json
{
  "keyword": "string (可选)",
  "category": "string (可选)",
  "format": "string (可选)"
}
```

**响应:** 匹配的图书数组

#### GET /api/books/categories - 获取图书分类

**权限:** 所有用户
**响应:** 分类字符串数组

#### GET /api/books/:id - 获取图书详情

**权限:**

- Admin/Root: 任意图书
- Contributor: 自己上传的图书 + 所属分组的图书
- User: 所属分组的图书
  **路径参数:** id - 图书ID
  **响应:** 图书详情

#### PUT /api/books/:id - 更新图书

**权限:** Admin/Root/Contributor
**路径参数:** id - 图书ID
**请求体:**

```json
{
  "title": "string (可选)",
  "author": "string (可选)",
  "description": "string (可选)",
  "isbn": "string (可选)",
  "category": "string (可选)"
}
```

**响应:** 更新后的图书

#### DELETE /api/books/:id - 删除图书

**权限:** Admin/Root/Contributor
**路径参数:** id - 图书ID
**响应:** 200 OK

#### GET /api/books/:id/read - 阅读图书

**权限:** 同获取图书详情
**路径参数:** id - 图书ID
**响应:**

```json
{
  "book": {...},
  "has_chapters": true/false,
  "supported": true/false,
  "total_chunks": 1,  // TXT无章节时
  "content": "string"  // TXT无章节时
}
```

#### GET /api/books/:id/file - 下载图书文件

**权限:** 同获取图书详情
**路径参数:** id - 图书ID
**响应:** 图书文件

#### GET /api/books/:id/resources/:filename - 获取图书资源

**权限:** 同获取图书详情
**路径参数:**

- id - 图书ID
- filename - 资源文件名
  **响应:** 资源文件

#### GET /api/books/:id/chunks/:chunk\_index - 获取TXT分块

**权限:** 同获取图书详情
**路径参数:**

- id - 图书ID
- chunk\_index - 分块索引
  **响应:**

```json
{
  "chunk_index": 0,
  "total_chunks": 10,
  "content": "string"
}
```

#### GET /api/books/:id/chapters - 获取图书章节列表

**权限:** 同获取图书详情
**路径参数:** id - 图书ID
**响应:** 章节数组

#### GET /api/books/:id/chapters/:chapter\_number - 获取章节详情

**权限:** 同获取图书详情
**路径参数:**

- id - 图书ID
- chapter\_number - 章节序号
  **响应:** 章节详情

***

### 分组接口

#### GET /api/groups - 获取分组列表

**权限:**

- Admin/Root: 所有分组
- 其他用户: 自己所属的分组
  **响应:** 分组数组

#### POST /api/groups - 创建分组

**权限:** Admin/Root/Contributor
**请求体:**

```json
{
  "name": "string",
  "description": "string (可选)"
}
```

**响应:** 201 创建成功

#### GET /api/groups/:id - 获取分组详情

**权限:**

- Admin/Root: 任意分组
- 其他用户: 自己所属的分组
  **路径参数:** id - 分组ID
  **响应:** 分组详情

#### PUT /api/groups/:id - 更新分组

**权限:** Admin/Root 或分组创建者
**路径参数:** id - 分组ID
**请求体:**

```json
{
  "name": "string (可选)",
  "description": "string (可选)"
}
```

**响应:** 更新后的分组

#### DELETE /api/groups/:id - 删除分组

**权限:** Admin/Root 或分组创建者
**路径参数:** id - 分组ID
**响应:** 200 OK

#### POST /api/groups/:id/users - 添加用户到分组

**权限:** Admin/Root 或分组创建者
**路径参数:** id - 分组ID
**请求体:**

```json
{
  "user_id": 1
}
```

**响应:** 201 创建成功

#### GET /api/groups/:id/users - 获取分组用户

**权限:** 分组成员
**路径参数:** id - 分组ID
**响应:** 用户数组

#### DELETE /api/groups/:id/users/:user\_id - 从分组移除用户

**权限:** Admin/Root 或分组创建者
**路径参数:**

- id - 分组ID
- user\_id - 用户ID
  **响应:** 200 OK

#### POST /api/groups/:id/books - 添加图书到分组

**权限:** Admin/Root 或分组创建者
**路径参数:** id - 分组ID
**请求体:**

```json
{
  "book_id": 1
}
```

**响应:** 201 创建成功

#### GET /api/groups/:id/books - 获取分组图书

**权限:** 分组成员
**路径参数:** id - 分组ID
**响应:** 图书数组

#### DELETE /api/groups/:id/books/:book\_id - 从分组移除图书

**权限:** Admin/Root 或分组创建者
**路径参数:**

- id - 分组ID
- book\_id - 图书ID
  **响应:** 200 OK

#### POST /api/groups/:id/invite-codes - 创建邀请码

**权限:** Admin/Root 或分组创建者
**路径参数:** id - 分组ID
**请求体:**

```json
{
  "max_users": 10 (可选),
  "expires_in_days": 7 (可选)
}
```

**响应:** 201 创建成功，返回邀请码信息

#### GET /api/groups/:id/invite-codes - 获取分组邀请码列表

**权限:** 分组成员
**路径参数:** id - 分组ID
**响应:** 邀请码数组

#### PUT /api/groups/:id/invite-codes/:code\_id - 停用邀请码

**权限:** Admin/Root 或邀请码创建者
**路径参数:**

- id - 分组ID
- code\_id - 邀请码ID
  **响应:** 更新后的邀请码

#### POST /api/groups/join-with-invite - 使用邀请码加入分组

**权限:** 所有用户
**请求体:**

```json
{
  "code": "ABCD1234"
}
```

**响应:** 201 加入成功

***

### 用户接口

#### GET /api/users - 获取用户列表

**权限:** Admin/Root
**响应:** 用户数组

#### POST /api/users - 创建用户

**权限:** Admin/Root

- Admin 只能创建 user/contributor 角色
- Root 可以创建所有角色
  **请求体:**

```json
{
  "username": "string",
  "password": "string",
  "role": "user/contributor/admin/root"
}
```

**响应:** 201 创建成功

#### PUT /api/users/me/profile - 更新个人资料

**权限:** 所有用户
**请求体:**

```json
{
  "password": "string (可选)"
}
```

**响应:** 更新后的用户信息

#### GET /api/users/:id - 获取用户详情

**权限:** Admin/Root 或用户本人
**路径参数:** id - 用户ID
**响应:** 用户详情

#### PUT /api/users/:id/role - 更新用户角色

**权限:**

- Root: 可以修改所有用户的角色
- Admin: 可以修改 user/contributor 的角色，但不能修改 Admin/Root 的角色
  **路径参数:** id - 用户ID
  **请求体:**

```json
{
  "role": "string"
}
```

**响应:** 更新后的用户

#### PUT /api/users/:id/password - 更新用户密码

**权限:** Admin/Root 或用户本人

- Admin 不能修改 Root 的密码
  **路径参数:** id - 用户ID
  **请求体:**

```json
{
  "password": "string"
}
```

**响应:** 200 OK

#### PUT /api/users/:id/disabled - 启用/禁用用户

**权限:** Admin/Root

- 不能禁用 Root 用户
- 不能禁用自己
  **路径参数:** id - 用户ID
  **请求体:**

```json
{
  "disabled": true/false
}
```

**响应:** 更新后的用户

#### DELETE /api/users/:id - 删除用户

**权限:** Admin/Root

- 不能删除 Root 用户
- 不能删除自己
  **路径参数:** id - 用户ID
  **响应:** 200 OK

***

### 设置接口

#### GET /api/settings - 获取系统设置

**权限:** Admin
**响应:**

```json
{
  "registration_enabled": true/false
}
```

#### PUT /api/settings/:key - 更新系统设置

**权限:** Admin
**路径参数:** key - 设置键 (如: registration\_enabled)
**请求体:**

```json
{
  "value": "string"
}
```

**响应:** 更新后的设置

## 数据库表结构

### users (用户表)

| 字段             | 类型       | 说明                                 |
| -------------- | -------- | ---------------------------------- |
| id             | i32      | 主键                                 |
| username       | String   | 用户名                                |
| password\_hash | Text     | 密码哈希                               |
| role           | String   | 用户角色 (user/contributor/admin/root) |
| disabled       | bool     | 是否禁用                               |
| created\_at    | DateTime | 创建时间                               |
| updated\_at    | DateTime | 更新时间                               |

### books (图书表)

| 字段          | 类型              | 说明                               |
| ----------- | --------------- | -------------------------------- |
| id          | i32             | 主键                               |
| title       | String          | 书名                               |
| author      | Option\<String> | 作者                               |
| description | Option\<String> | 描述                               |
| file\_path  | String          | 文件存储路径                           |
| file\_type  | String          | 文件类型 (epub/txt/pdf等)             |
| file\_size  | i64             | 文件大小(字节)                         |
| file\_hash  | Option\<String> | 文件SHA256哈希                       |
| cover\_path | Option\<String> | 封面路径                             |
| isbn        | Option\<String> | ISBN编号                           |
| category    | Option\<String> | 分类                               |
| created\_by | Option\<i32>    | 创建者用户ID                          |
| status      | String          | 审核状态 (pending/approved/rejected) |
| created\_at | DateTime        | 创建时间                             |
| updated\_at | DateTime        | 更新时间                             |

### book\_uploaders (图书上传者表)

| 字段          | 类型       | 说明      |
| ----------- | -------- | ------- |
| id          | i32      | 主键      |
| book\_id    | i32      | 图书ID    |
| user\_id    | i32      | 上传者用户ID |
| created\_at | DateTime | 创建时间    |

### chapters (章节表)

| 字段              | 类型       | 说明           |
| --------------- | -------- | ------------ |
| id              | i32      | 主键           |
| book\_id        | i32      | 所属图书ID       |
| chapter\_number | i32      | 章节序号         |
| title           | String   | 章节标题         |
| content         | String   | 章节内容(HTML格式) |
| created\_at     | DateTime | 创建时间         |
| updated\_at     | DateTime | 更新时间         |

### comments (评论表)

| 字段          | 类型           | 说明                               |
| ----------- | ------------ | -------------------------------- |
| id          | i32          | 主键                               |
| book\_id    | Option\<i32> | 图书ID                             |
| chapter\_id | Option\<i32> | 章节ID                             |
| user\_id    | i32          | 用户ID                             |
| content     | String       | 评论内容                             |
| status      | String       | 审核状态 (pending/approved/rejected) |
| created\_at | DateTime     | 创建时间                             |
| updated\_at | DateTime     | 更新时间                             |

### groups (分组表)

| 字段          | 类型              | 说明      |
| ----------- | --------------- | ------- |
| id          | i32             | 主键      |
| name        | String          | 分组名称    |
| description | Option\<String> | 分组描述    |
| created\_by | Option\<i32>    | 创建者用户ID |
| created\_at | DateTime        | 创建时间    |
| updated\_at | DateTime        | 更新时间    |

### user\_groups (用户-分组关系表)

| 字段          | 类型       | 说明   |
| ----------- | -------- | ---- |
| id          | i32      | 主键   |
| user\_id    | i32      | 用户ID |
| group\_id   | i32      | 分组ID |
| created\_at | DateTime | 创建时间 |

### book\_groups (图书-分组关系表)

| 字段          | 类型       | 说明   |
| ----------- | -------- | ---- |
| id          | i32      | 主键   |
| book\_id    | i32      | 图书ID |
| group\_id   | i32      | 分组ID |
| created\_at | DateTime | 创建时间 |

### invite\_codes (邀请码表)

| 字段          | 类型                | 说明      |
| ----------- | ----------------- | ------- |
| id          | i32               | 主键      |
| code        | String            | 邀请码     |
| group\_id   | i32               | 所属分组ID  |
| created\_by | i32               | 创建者用户ID |
| max\_users  | Option\<i32>      | 最大使用人数  |
| used\_count | i32               | 已使用次数   |
| expires\_at | Option\<DateTime> | 过期时间    |
| is\_active  | bool              | 是否激活    |
| created\_at | DateTime          | 创建时间    |
| updated\_at | DateTime          | 更新时间    |

### settings (设置表)

| 字段          | 类型       | 说明     |
| ----------- | -------- | ------ |
| key         | String   | 主键，设置键 |
| value       | String   | 设置值    |
| created\_at | DateTime | 创建时间   |
| updated\_at | DateTime | 更新时间   |

## 系统设置项

| 设置键                     | 说明          | 默认值   |
| ----------------------- | ----------- | ----- |
| registration\_enabled   | 是否允许用户注册    | true  |
| allow\_uploader\_edit   | 是否允许上传者修改书籍 | true  |
| allow\_uploader\_delete | 是否允许上传者删除书籍 | true  |
| enable\_upload\_review  | 是否开启上传审核    | false |
| allow\_comments         | 是否允许评论      | true  |
| enable\_comment\_review | 是否开启评论审核    | false |

