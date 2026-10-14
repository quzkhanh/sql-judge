-- Edge fixture: zero tasks, all NULL estimates, tie dates, duplicate keys, deeper ancestry.
INSERT INTO users (id,name,is_active,joined_at) VALUES
 (11,'U11',TRUE,'2025-01-01'),(12,'U12',TRUE,'2025-01-01'),
 (13,'U13',FALSE,'2025-01-01'),(14,'U14',TRUE,'2025-01-01');
INSERT INTO projects (id,name,owner_id,status) VALUES
 (110,'Zero',11,'ACTIVE'),(120,'Deep',11,'ACTIVE'),
 (130,'Frozen',12,'ARCHIVED'),(140,'Unused',NULL,'ACTIVE'),
 (150,'Null Estimates',13,'ACTIVE'),(160,'Only Open',14,'ACTIVE');
INSERT INTO tasks (id,project_id,parent_task_id,title,status,priority,created_at,completed_at,estimated_hours,capacity) VALUES
 (1100,120,NULL,'Root','AVAILABLE',1,'2026-01-01',NULL,0,1),
 (1101,120,1100,'Child','AVAILABLE',1,'2026-01-01',NULL,0,1),
 (1102,120,1101,'Grandchild','DONE',1,'2026-01-01','2026-01-02',NULL,1),
 (1103,120,1101,'Grandchild B','AVAILABLE',1,'2026-01-01',NULL,50,2),
 (1104,120,1103,'Deep leaf','IN_PROGRESS',9,'2026-01-01',NULL,10,1),
 (1200,130,NULL,'Old','DONE',10,'2026-01-01','2026-01-02',9,1),
 (1300,150,NULL,'Unknown',NULL,3,'2026-01-09',NULL,NULL,1),
 (1301,150,NULL,'No estimate','AVAILABLE',3,'2026-01-09',NULL,NULL,1),
 (1400,160,NULL,'Open 1','AVAILABLE',2,'2026-01-08',NULL,10,2),
 (1401,160,NULL,'Open 2','IN_PROGRESS',2,'2026-01-08',NULL,10,1);
INSERT INTO assignments (id,task_id,user_id,status,assigned_at) VALUES
 (100,1100,11,'ACTIVE','2026-01-02'),
 (101,1100,12,'ACTIVE','2026-01-02'),
 (102,1101,11,'CANCELLED','2026-01-02'),
 (103,1103,12,'DONE','2026-01-03'),
 (104,1103,12,'ACTIVE','2026-01-03'),
 (105,1103,12,'ACTIVE','2026-01-03'),
 (106,1400,11,'ACTIVE','2026-01-05'),
 (107,1401,14,'CANCELLED','2026-01-05'),
 (108,1401,14,'DONE','2026-01-05');
INSERT INTO work_logs (id,task_id,user_id,logged_on,hours) VALUES
 (100,1100,11,'2026-01-01',NULL),
 (101,1101,11,'2026-01-01',2),
 (102,1101,11,'2026-01-09',3),
 (103,1102,11,'2026-01-17',0),
 (104,1103,12,'2026-01-03',10),
 (105,1103,12,'2026-01-03',10),
 (106,1104,12,'2026-02-10',50),
 (107,1200,13,'2026-04-01',NULL),
 (108,1400,14,'2026-02-03',1),
 (109,1401,14,'2026-02-17',1);
INSERT INTO imports (id,project_id,started_at,status) VALUES
 (10,120,'2026-01-01','NEW'),(11,120,'2026-01-02','DONE'),(12,110,'2026-01-03','DONE');
INSERT INTO import_staging (id,import_id,external_key,user_id,source_row_number,payload_status) VALUES
 (100,10,'same',11,1,'DONE'),(101,10,'same',12,1,'DONE'),
 (102,10,'same',11,3,'INVALID'),(103,10,NULL,NULL,4,NULL),
 (104,10,NULL,11,5,'AVAILABLE'),(105,10,'unique',11,6,NULL),
 (106,11,'same',12,2,'AVAILABLE'),(107,11,'same',12,4,'IN_PROGRESS'),
 (108,11,'other',12,1,'DONE');
INSERT INTO payments (id,project_id,amount,paid_at,status) VALUES
 (100,120,0,'2026-01-01','PAID'),(101,120,0,'2026-01-02','REFUNDED'),
 (102,120,NULL,'2026-02-03','PAID'),(103,120,150,'2026-02-04','PENDING'),
 (104,150,5,'2026-03-01','REFUNDED'),(105,160,10,'2026-04-01','PAID');
