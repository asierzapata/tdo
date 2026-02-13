# `tdo` Command Cheat Sheet

## Capture

| Command                                | Description                |
| -------------------------------------- | -------------------------- |
| `tdo add "task"`                       | Add to Inbox               |
| `tdo add "task" --today`               | Add to Today               |
| `tdo add "task" --evening`             | Add to Today (evening)     |
| `tdo add "task" --someday`             | Add to Someday             |
| `tdo add "task" --anytime`             | Add to Anytime             |
| `tdo add "task" --when friday`         | Schedule for specific date |
| `tdo add "task" --deadline 2025-03-01` | Set hard deadline          |
| `tdo add "task" -p project-slug`       | Add to project             |
| `tdo add "task" -t tag1 -t tag2`       | Add with tags              |
| `tdo add "task" -n "some notes"`       | Add with notes             |

## View

| Command              | Shows                         |
| -------------------- | ----------------------------- |
| `tdo`                | Today (default)               |
| `tdo today`          | Today + overdue               |
| `tdo inbox`          | Uncategorized tasks           |
| `tdo upcoming`       | Future-dated, grouped by date |
| `tdo anytime`        | No date, not someday          |
| `tdo someday`        | Explicitly deferred           |
| `tdo logbook`        | Completed (last 14 days)      |
| `tdo trash`          | Soft-deleted                  |
| `tdo all`            | Everything active             |
| `tdo projects`       | List all projects             |
| `tdo project <slug>` | Tasks in specific project     |

## Act on Tasks

| Command                  | Description             |
| ------------------------ | ----------------------- |
| `tdo done <id>`          | Complete task           |
| `tdo done "fuzzy match"` | Complete by title match |
| `tdo edit <id>`          | Edit in `$EDITOR`       |
| `tdo delete <id>`        | Move to trash           |
| `tdo restore <id>`       | Restore from trash      |

## Move / Schedule

| Command                         | Description        |
| ------------------------------- | ------------------ |
| `tdo today <id>`                | Move to Today      |
| `tdo someday <id>`              | Move to Someday    |
| `tdo anytime <id>`              | Move to Anytime    |
| `tdo inbox <id>`                | Move back to Inbox |
| `tdo move <id> --when friday`   | Schedule for date  |
| `tdo move <id> -p project-slug` | Assign to project  |

## Projects

| Command                              | Description      |
| ------------------------------------ | ---------------- |
| `tdo project new "Name"`             | Create project   |
| `tdo project new "Name" --area work` | Create in area   |
| `tdo project done <slug>`            | Complete project |
| `tdo project delete <slug>`          | Delete project   |

## Flags Reference

| Flag                | Short | Description          |
| ------------------- | ----- | -------------------- |
| `--today`           |       | Schedule for today   |
| `--evening`         |       | Today, but evening   |
| `--someday`         |       | Defer to someday     |
| `--anytime`         |       | Available anytime    |
| `--when <date>`     | `-w`  | Schedule for date    |
| `--deadline <date>` | `-d`  | Hard due date        |
| `--project <slug>`  | `-p`  | Assign to project    |
| `--tag <name>`      | `-t`  | Add tag (repeatable) |
| `--notes "text"`    | `-n`  | Add notes            |
