# PostgreSQL CTE Mastery — 20 bài (Junior → Middle)

**Format:** `sqljudge.pack/v1` (cùng cấu trúc với pack mẫu `hr-sql-101`).
**Dialect:** PostgreSQL >= 15. **Số bài:** 20. **Tests:** 60 fixtures độc lập (basic hiện, edge ẩn, empty ẩn; empty có project nhưng không có task).
**Kiểu chấm:** `unordered`, strict column name/type, SELECT-only reference, không phụ thuộc thứ tự rows.

## Bài học

| #   | Chủ đề                                | Mức    |
| --- | ------------------------------------- | ------ |
| 01  | Available Tasks in Active Projects    | easy   |
| 02  | Estimates Above Project Average       | easy   |
| 03  | Project Status Dashboard              | easy   |
| 04  | Projects Above Global Task Average    | easy   |
| 05  | Multi-Task Assignment Load            | easy   |
| 06  | Projects Without Completed Tasks      | medium |
| 07  | Available Assignment Slots            | medium |
| 08  | Largest Estimated Task per Project    | medium |
| 09  | Latest Assignment Audit               | medium |
| 10  | Second Newest Task                    | medium |
| 11  | Global Running Work Hours             | medium |
| 12  | Monthly Net Payments                  | medium |
| 13  | Find Projects Over Budgeted Hours     | medium |
| 14  | Deterministic Import Deduplication    | medium |
| 15  | Import Quality & Duplicate Audit      | hard   |
| 16  | Project User Effort Leaderboard       | hard   |
| 17  | Detect Inactivity Gaps Over 7 Days    | hard   |
| 18  | Traverse a Task Hierarchy             | hard   |
| 19  | Roll Up Descendant Estimated Hours    | hard   |
| 20  | Pick Next Assignable Task per Project | hard   |

## Hướng dẫn luyện

1. Xác định **result grain**: một dòng trong output đại diện cho gì.
2. Dự đoán kết quả trên fixture `basic` trước khi viết SQL.
3. Tự triển khai `WITH ... AS` (bài 18–19: `WITH RECURSIVE`) và đối chiếu `edge` / `empty`.
4. Tự tìm lỗi `NULL`, nhân bản JOIN, tie-breaker, ngày trùng, 0 bản ghi, phân cấp nhiều tầng.
5. `reference.sql` là lời giải của judge, **không nên mở trước khi tự giải**.

## Import / security

- File `pack.json` nằm ở ROOT ZIP, không có thư mục bao ngoài.
- Mỗi bài trỏ relative tới `reference.sql` và `tests/*.sql` trong folder riêng.
- File `schema/001_tables.sql` nạp trước; mỗi fixture nạp vào một DB/test transaction sạch.
- Fixture `basic` và `edge` độc lập: không chạy chung vào cùng database đã có dữ liệu.
- Toàn bộ là bài đọc SELECT, không có yêu cầu thay đổi dữ liệu/DDL khi nộp.
- `WITH` không thể cưỡng chế chỉ bằng đối chiếu kết quả; statement quy định để phục vụ học tập.
- Contract dựa trên ZIP mẫu do người dùng cung cấp; importer cần kiểm tra thực tế trên PostgreSQL trước khi cài.
