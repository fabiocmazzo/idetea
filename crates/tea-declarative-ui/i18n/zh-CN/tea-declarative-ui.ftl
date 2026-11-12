## 文档错误（显示在面板里，给用户和智能体修正）
ui-too-big = 文档有 { $size } 字节；最大是 { $max }。
ui-not-json = 文件不是有效的JSON：{ $error }
ui-not-object = 文档必须是JSON对象。
ui-wrong-format = "format" 必须是 "idetea-ui"（实际是 { $found }）。
ui-unsupported-version = 不支持格式版本 { $version }（此IDE认识 { $supported }）。
ui-missing = 缺少字段 "{ $field }"。
ui-wrong-type = "{ $field }" 的类型必须是 { $expected }。
ui-too-many = "{ $field }" 的项超过 { $max } 个。
ui-too-long = "{ $field }" 长于 { $max } 字节。
ui-bad-id = "{ $field }"：id无效（"{ $id }"）。请用字母、数字、"_" 和 "-"（最多64个）。
ui-duplicate-id = id重复："{ $id }"。
ui-bad-path = "{ $field }"："{ $path }" 不是有效路径（例如 "/name"）。
ui-bad-file-path = "{ $field }"："{ $path }" 必须是相对于项目根的路径，且不能有 ".."。
ui-data-too-big = "data" 大于 { $max } 字节。
ui-data-too-deep = "data" 超过 { $max } 层。

## 警告（文档仍会打开）
ui-unknown-field = 已忽略未知字段："{ $field }"。
ui-unknown-value = "{ $field }"：未知值 "{ $value }"，改用默认值。
ui-unknown-component = 此版本不支持组件 "{ $component }"。
ui-link-error = 无法打开 { $url }：{ $error }
ui-link-ignored = 已忽略链接（只接受网页、项目文件和idetea:config）：{ $url }
ui-comment-label = 给智能体的评论（可选）
ui-comment-placeholder = 会随你点的按钮一起送出
ui-copy = 复制
ui-copied = 已复制。
ui-steps-missing = { $count ->
   *[other] 还有 { $count } 个步骤待勾选。
}
ui-step-optional = （可选）
ui-requires-unbound = "{ $field }"："{ $path }" 不是任何组件的 "value"；已忽略。
ui-fields-missing = 还要填：{ $fields }
ui-too-many-images = 文档里的图像超过 { $max } 张。
ui-image-loading = { $path }：正在加载…
ui-image-error = 无法读取：{ $error }
ui-image-not-file = 不是文件（不跟随符号链接）
ui-image-too-big = 大于 { $max } MB
ui-image-too-large = { $width }×{ $height } px；每边最大 { $max }
ui-image-total = 文档里的图像超过 { $max } 百万像素
ui-too-many-sounds = 文档里的声音超过 { $max } 个。
ui-audio-play = 播放
ui-audio-stop = 停止
ui-audio-info = { $seconds } 秒 · { $rate } Hz · { $channels ->
       *[other] { $channels } 声道
    } · { $bits } 位 · { $peak }
ui-audio-peak = 峰值 { $db } dBFS
ui-audio-silent = 静音
ui-audio-loop = 循环中
ui-audio-clips = 达到0 dBFS：可能会削波。
ui-audio-extension = 不是声音（接受：{ $list }）。
ui-audio-too-big = 大于 { $max } MB：可以播放，但没有时长和波形。
ui-audio-unreadable = 没有时长或波形（{ $error }）；仍然可以播放。
ui-audio-no-player = 找不到声音播放器（pw-play、paplay、aplay或afplay）。
ui-audio-plugin = 插件面板不播放声音。
ui-sprite-pause = 暂停
ui-sprite-play = 播放动画
ui-sprite-step = 下一帧
ui-sprite-info = 第 { $frame } 帧，共 { $count } 帧 · { $fps } fps
ui-sprite-frame = { $field }：frame_width和frame_height要一起写。
ui-sprite-too-small = 整张图（{ $width }×{ $height }）比一帧还小。
ui-open-blocked = { $path }：idetea:open链接只能打开 { $list }。
ui-open-not-file = { $path } 不是普通文件（不存在、是文件夹或符号链接）。
ui-select-text = 选择文本
ui-select-done = 显示格式化结果
