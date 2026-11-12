## Git
cmd-git-focus = Git: 显示更改
cmd-git-refresh = Git: 刷新
cmd-git-stage-all = Git: 全部暂存
cmd-git-unstage-all = Git: 全部取消暂存
cmd-git-commit = Git: 提交（聚焦到消息）
cmd-git-pull = Git: 拉取（下载并合并）
cmd-git-push = Git: 推送（发送提交）
cmd-git-fetch = Git: 获取（查看远程是否有更新）
git-panel = Git
cancel = 取消
copied = 已复制：{ $text }
git-notice = Git：{ $message }
git-wait = Git：请稍候，仍在 { $running }
git-fetching = 正在获取…
git-pulling = 正在拉取…
git-pushing = 正在推送…
git-publishing = 正在发布…
git-no-repo-notice = Git：此文件夹不在仓库中
git-no-repo = 此文件夹不在git仓库里。
git-need-message = Git：请写提交消息
git-nothing-staged = Git：没有已暂存的内容（用 + 添加）
git-detached-pull = Git：处于游离HEAD：请检出一个分支再拉取
git-detached-push = Git：处于游离HEAD：请检出一个分支再推送
git-no-upstream = Git：此分支没有跟踪远程（推送以发布它）
git-no-remote = Git：仓库没有远程（git remote add origin <url>）
git-discarded = 已放弃 { $path } 中的更改
git-discard-untracked = 该文件是新建的（未跟踪），会进回收站。
git-discard-tracked = 未暂存的更改会丢失；文件会回到已暂存的内容，或上次提交。
git-discard-title = 放弃“{ $path }”中的更改？
git-discard = 放弃
git-publish-title = 发布分支“{ $branch }”？
git-publish-detail = 它在 { $remote } 上还不存在。这次推送会在那里创建分支，并开始跟踪（git push -u { $remote } { $branch }）。
git-publish = 发布
git-fetch-tip = 获取：查看远程是否有更新
git-pull-tip = 拉取：下载并合并{ $count }
git-push-tip = 推送：发送提交{ $count }
git-refresh = 刷新
git-commit-placeholder = 提交消息（Enter）
git-staged = 已暂存
git-changes = 更改
git-unstage-all = 全部取消暂存
git-stage-all = 全部暂存
git-unstage = 取消暂存
git-stage = 暂存
git-nothing-to-commit = 没有可提交的内容。
git-open-file = 打开文件
git-view-diff = 查看差异
git-discard-menu = 放弃更改…
git-copy-path = 复制路径
git-copy-relative = 复制相对路径
git-select-file = 选一个文件查看差异。
git-diff-truncated = … 差异在 { $count } 行处截断
git-no-diff = 没有差异（二进制文件，或只改了权限）。
git-run-error = 无法运行git：{ $error }
git-unknown-error = 未知错误
git-trash-error = 无法移到回收站：{ $error }
git-timeout = 和远程通信超时（{ $seconds } 秒）
git-explain-rejected = 远程有你没有的提交：请先拉取
git-explain-diverged = 本地分支和远程分支已经分叉：请设置pull.rebase（或在终端里解决）
git-explain-conflict = 拉取出现冲突：解决标出的文件并提交
git-explain-overwritten = 本地更改会被覆盖：拉取前请提交或放弃
git-explain-auth = 身份验证失败：请配置凭据助手（例如gh auth setup-git）
git-explain-ssh = ssh拒绝了连接：请把密钥载入ssh-agent（ssh-add），或在终端里确认服务器
git-explain-network = 无法到达远程（网络或地址）
git-fetch-done = 获取完成
git-up-to-date = 已经是最新
git-pull-done = 拉取完成
git-published = 分支 { $branch } 已发布到 { $remote }
git-nothing-to-push = 没有要推送的
git-push-done = 推送完成

## 分支
cmd-git-checkout = Git: 切换分支
git-checkout-placeholder = 切换到分支…（或输入新名称）
git-create-branch = 创建分支
git-branch-current = 当前 · { $date }
git-switched = 已切换到分支 { $branch }
git-switch-failed = 无法切换到 { $branch }：{ $error }
git-explain-switch-overwritten = 未提交的更改和另一个分支冲突：请先提交或放弃

## 追溯
git-blame-uncommitted = 你 · 尚未提交
git-blame-detail = 
    { $author }，{ $ago }
    提交 { $hash }
git-blame-copy = 复制哈希
git-close = 关闭
git-ago-now = 刚刚
git-ago-minutes = 
    { $count ->
       *[other] { $count } 分钟前
    }
git-ago-hours = 
    { $count ->
       *[other] { $count } 小时前
    }
git-ago-days = 
    { $count ->
       *[other] { $count } 天前
    }
git-ago-months = 
    { $count ->
       *[other] { $count } 个月前
    }
git-ago-years = 
    { $count ->
       *[other] { $count } 年前
    }
