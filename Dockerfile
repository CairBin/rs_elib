# 构建阶段
FROM rust:latest as builder

# 设置工作目录
WORKDIR /app

# 复制Cargo.toml和Cargo.lock
COPY Cargo.toml Cargo.lock ./

# 创建一个空的main.rs文件来构建依赖
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs

# 构建依赖
RUN cargo build --release

# 复制源代码
COPY src ./src
COPY static ./static

# 构建应用
RUN cargo build --release

# 运行阶段
FROM debian:bullseye-slim

# 安装必要的依赖
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 从构建阶段复制编译好的二进制文件
COPY --from=builder /app/target/release/rs_elib .

# 复制静态文件
COPY --from=builder /app/static ./static

# 创建上传目录
RUN mkdir -p uploads

# 暴露端口
EXPOSE 3000

# 设置环境变量默认值
ENV JWT_SECRET="default_secret_change_in_production"

# 设置ENTRYPOINT和CMD
ENTRYPOINT ["./rs_elib"]
CMD []
