#!/bin/sh
# 基线 e2e 流水线脚本（github-activity 项目）
#
# 输入：未脱敏活动明细 CSV（tests/fixtures/github-activity/raw.csv）
# 输出：脱敏 + 排序的最终交付 CSV
#
# 1) 用户 ID 替换为去标识化序号（脱敏，去掉 user_id / login 列）
# 2) 按 用户序号 + 日期 排序
# 3) 输出标准表头
set -eu
printf 'user_seq,date,push_count,pr_count,issue_comments,pr_comments,pr_merges,commit_comments,bot\n' > "$2"
awk -F, 'NR > 1 {
  if (!($1 in seq)) { seq[$1] = ++n }
  print seq[$1] "," $3 "," $4 "," $5 "," $6 "," $7 "," $8 "," $9 "," $10
}' "$1" | LC_ALL=C sort -t, -k1,1n -k2,2 >> "$2"
