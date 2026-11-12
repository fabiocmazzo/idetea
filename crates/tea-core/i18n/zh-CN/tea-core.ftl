## 计数（引用面板）
count-references = 
    { $count ->
       *[other] { $count } 个引用
    }
count-definitions = 
    { $count ->
       *[other] { $count } 个定义
    }
count-implementations = 
    { $count ->
       *[other] { $count } 个实现
    }
count-types = 
    { $count ->
       *[other] { $count } 个类型
    }

## 符号（文件结构）
kind-module = 模块
kind-class = 类
kind-struct = 结构体
kind-interface = 接口/trait
kind-enum = 枚举
kind-variant = 变体
kind-method = 方法
kind-function = 函数
kind-constructor = 构造函数
kind-field = 字段
kind-property = 属性
kind-variable = 变量
kind-constant = 常量
kind-type-parameter = 类型参数
kind-block = 代码块
kind-other = 符号
visibility-public = 公共
visibility-private = 私有

## 配置与快捷键错误
chord-empty = 空快捷键
chord-unknown-modifier = 未知修饰键：{ $modifier }
chord-no-key = 快捷键里没有按键：{ $chord }
config-key-not-text = { $layer }：[keys] { $chord } 必须是文本，实际是 { $value }
config-keys-not-table = { $layer }：[keys] 必须是表
config-unknown-section = 未知的节：[{ $section }]
config-agents-project = { $layer }：[agents] 只在用户配置（~/.config/idetea/config.toml）里有效；这里已忽略
config-font-missing = [editor] font = "{ $name }"：未安装该字体，改用 { $default }
config-font-weight = [editor] font_weight = "{ $name }"：请用其中之一：{ $choices }
config-file-icons = [ui] file_icons = "{ $value }"：请用 "seti" 或 "none"
