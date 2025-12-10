# TODO for zako.json

## 支持在`zako.json`的配置中使用继承。

如

```json
{
    "group": "moe.fra",
    "artifact": "example",
    "version": "1.0.0",
    "options": {
        "debug":{
            "inherit": "fra.moe:zako@1.0.0#config//debug"
        }
    }
}
```

## 使用版本别名

使用版本别名

```json
{
    "dependencies": {
        // 定义别名 "std"，指向 zako 的某个版本
        "std": "fra.moe:zako@^1.5"
    },
    "options": {
        "debug": {
            // 使用别名引用，不写版本号
            "inherit": "@std#config//debug"
        }
    }
}
```

## 惊群效应

Resolve 惊群效应

## Fan-out 问题

Resolve fan-out
