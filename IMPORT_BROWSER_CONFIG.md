# 浏览器导入配置 JSON 说明

控制器顶部的“导入浏览器”按钮读取 JSON 文件，并调用 `import_profiles` 导入浏览器环境。其他程序可以按下面格式生成配置。

## 推荐格式

```json
{
  "exportedAt": "2026-05-28T12:00:00Z",
  "projects": [
    { "id": "default", "name": "默认项目" },
    { "id": "jp-shop", "name": "日本项目" }
  ],
  "profiles": [
    {
      "id": "external-001",
      "projectId": "jp-shop",
      "name": "jp-shop-1",
      "note": "外部程序生成",
      "proxy": "socks5://user:pass@host:port",
      "proxyProtocol": "SOCKS5",
      "proxyIp": "59.147.25.114",
      "proxyCountry": "JP",
      "locale": "ja-JP",
      "timezone": "Asia/Tokyo",
      "seed": "74100",
      "screenMode": "random",
      "screenWidth": 1600,
      "screenHeight": 900,
      "fontMode": "random",
      "hardwareConcurrency": 8,
      "mediaDevices": "random",
      "deviceMemory": 8,
      "webglImage": "random",
      "gpuRenderer": "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0)",
      "injectionScripts": [
        "E:\\\\scripts\\\\example.user.js"
      ],
      "autoLocale": false,
      "autoTimezone": true
    }
  ]
}
```

## 字段要点

- `projects` 可选；如果 `profiles[].projectId` 不存在，会自动放到当前项目。
- `profiles[].id` 导入时会重新生成，外部程序可以随便填一个非空字符串。
- `name` 是浏览器名称；克隆时会按 `name-1`、`name-2` 递增。
- `proxy` 支持 `socks5://user:pass@host:port`、`http://host:port`、`host:port:user:pass` 等常见格式。
- `locale` 和 `timezone` 是分开的；`autoLocale`、`autoTimezone` 也分别控制启动时是否按代理自动同步。
- `seed` 是指纹种子；为空会自动生成。
- `screenMode/fontMode/mediaDevices/webglImage` 设置为 `random` 时，保存或克隆会重新随机对应项。
- `injectionScripts` 是永久注入脚本路径数组，复制/克隆浏览器时会一起带走。

## 最小可导入示例

```json
{
  "profiles": [
    {
      "id": "p1",
      "projectId": "default",
      "name": "test-1",
      "seed": "12345",
      "proxy": "",
      "locale": "zh-CN",
      "timezone": "Asia/Shanghai",
      "screenWidth": 1920,
      "screenHeight": 1080
    }
  ]
}
```
