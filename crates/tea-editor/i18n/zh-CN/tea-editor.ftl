## 编辑器、图像与PDF
binary-file = 这是二进制文件，不是文本
not-valid-in = 文件不是有效的 { $encoding }
not-in-encoding = “{ $ch }”（第 { $line } 行第 { $column } 列）在 { $encoding } 中不存在；另存为 UTF-8 才能保留
untitled = 未命名
image-info = { $width } × { $height } px · { $size } · 滚轮缩放，拖动平移
image-error = 无法读取图像：{ $error }
pdf-error = 无法打开PDF：{ $error }
pdf-page = 第 { $current } 页，共 { $total } 页
pdf-fit = 适合
pdf-encrypted = 有密码的PDF（暂不支持）
pdf-invalid = 文件无效或已损坏

language-image = 图像
language-light = { $language }（轻量模式）

## Git标记
hunk-added = 
    { $count ->
       *[other] 新增 { $count } 行
    }
hunk-modified = 
    { $count ->
       *[other] 更改了 { $count } 行；原来是：
    }
hunk-deleted = 
    { $count ->
       *[other] 删除了 { $count } 行：
    }
hunk-added-note = 不在上次提交里。
hunk-revert = 还原更改
image-saved = 保存于 { $time }

## 表格视图 (RFC 0020)
table-show-text = 文本
table-filter = 筛选
table-rows = { $count } 行
table-rows-filtered = { $count } 行中的 { $shown } 行
table-sorted = 按 { $column } 排序 { $arrow }
table-invalid-json = 不是 JSON：{ $error }
table-broken-row = 这一行无法读取，请在文本中编辑。
table-out-of-range = 这个单元格不存在。
table-header-row = 第一行是表头
table-rows-picked = 已选 { $count } 行
