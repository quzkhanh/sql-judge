INSERT INTO users (id,name,is_active,joined_at) VALUES
 (1,'An',TRUE,'2025-01-01'),(2,'Binh',TRUE,'2025-02-10'),
 (3,'Chi',TRUE,'2025-03-01'),(4,'Dung',TRUE,'2025-04-01'),
 (5,'Em',FALSE,'2025-05-01'),(6,'Phuc',TRUE,'2025-06-01');
INSERT INTO projects (id,name,owner_id,status) VALUES
 (10,'Atlas',1,'ACTIVE'),(20,'Beacon',2,'ACTIVE'),
 (30,'Archive',3,'ARCHIVED'),(40,'Empty',NULL,'ACTIVE'),(50,'No Tasks',1,'ACTIVE');
INSERT INTO tasks (id,project_id,parent_task_id,title,status,priority,created_at,completed_at,estimated_hours,capacity) VALUES
 (100,10,NULL,'Build API','AVAILABLE',5,'2026-01-03',NULL,8,2),
 (101,10,100,'Add auth','IN_PROGRESS',7,'2026-01-04',NULL,5,1),
 (102,10,100,'Add tests','DONE',4,'2026-01-05','2026-01-09',3,1),
 (103,10,101,'Token rotation','AVAILABLE',9,'2026-01-05',NULL,13,2),
 (104,10,NULL,'Spike','DONE',1,'2026-01-03','2026-01-10',NULL,1),
 (105,10,NULL,'Code review','AVAILABLE',8,'2026-01-08',NULL,8,1),
 (200,20,NULL,'Index search','AVAILABLE',6,'2026-01-02',NULL,7,1),
 (201,20,200,'Analyze plan','IN_PROGRESS',10,'2026-01-06',NULL,4,2),
 (202,20,NULL,'Release','DONE',2,'2026-01-06','2026-01-20',8,2),
 (203,20,NULL,'Triage','AVAILABLE',1,'2026-01-06',NULL,NULL,1),
 (204,20,NULL,'Review locks','AVAILABLE',3,'2026-01-01',NULL,7,2),
 (300,30,NULL,'Legacy export','DONE',3,'2026-01-07','2026-01-07',20,1),
 (301,30,NULL,'Old fix','AVAILABLE',7,'2026-01-09',NULL,6,1);
INSERT INTO assignments (id,task_id,user_id,status,assigned_at) VALUES
 (1,100,1,'ACTIVE','2026-01-03'),(2,100,2,'CANCELLED','2026-01-05'),
 (3,101,1,'ACTIVE','2026-01-04'),(4,101,3,'ACTIVE','2026-01-06'),
 (5,102,2,'DONE','2026-01-05'),(6,103,2,'ACTIVE','2026-01-07'),
 (7,103,4,'ACTIVE','2026-01-07'),(8,200,2,'CANCELLED','2026-01-03'),
 (9,201,3,'ACTIVE','2026-01-05'),(10,201,4,'ACTIVE','2026-01-06'),
 (11,202,1,'DONE','2026-01-07'),(12,301,5,'ACTIVE','2026-01-06'),
 (13,204,5,'ACTIVE','2026-01-05'),(14,204,6,'ACTIVE','2026-01-05'),
 (15,105,1,'CANCELLED','2026-01-06'),(16,105,1,'ACTIVE','2026-01-08');
INSERT INTO work_logs (id,task_id,user_id,logged_on,hours) VALUES
 (1,100,1,'2026-01-02',2),(2,100,1,'2026-01-03',3),
 (3,101,2,'2026-01-03',4),(4,102,1,'2026-01-10',6),
 (5,103,3,'2026-01-10',7),(6,200,2,'2026-02-01',4),
 (7,201,2,'2026-02-03',5),(8,201,4,'2026-02-10',2),
 (9,202,5,'2026-02-10',3),(10,300,1,'2026-02-16',20),
 (11,301,6,'2026-02-18',NULL),(12,103,3,'2026-01-22',2),
 (13,200,2,'2026-02-01',1),(14,103,4,'2026-01-03',2),
 (15,200,2,'2026-02-11',30);
INSERT INTO imports (id,project_id,started_at,status) VALUES
 (1,10,'2026-01-01','DONE'),(2,20,'2026-02-01','DONE'),(3,40,'2026-02-02','NEW');
INSERT INTO import_staging (id,import_id,external_key,user_id,source_row_number,payload_status) VALUES
 (1,1,'A',1,1,'AVAILABLE'),(2,1,'A',1,2,'AVAILABLE'),
 (3,1,'B',2,3,'DONE'),(4,1,NULL,3,4,'AVAILABLE'),
 (5,1,'C',NULL,5,'IN_PROGRESS'),(6,1,'D',3,6,'UNKNOWN'),
 (7,2,'X',1,1,'AVAILABLE'),(8,2,'X',3,3,'UNKNOWN'),
 (9,2,'Y',4,2,'IN_PROGRESS'),(10,2,'Y',4,2,'IN_PROGRESS'),
 (11,2,'Z',5,5,NULL),(12,2,NULL,NULL,6,NULL);
INSERT INTO payments (id,project_id,amount,paid_at,status) VALUES
 (1,10,100,'2026-01-05','PAID'),(2,10,20,'2026-01-06','REFUNDED'),
 (3,10,999,'2026-01-07','PENDING'),(4,10,150,'2026-02-09','PAID'),
 (5,10,NULL,'2026-02-10','REFUNDED'),(6,10,200,'2026-04-01','PAID'),
 (7,20,30,'2026-02-02','PAID'),(8,20,60,'2026-03-10','PAID'),
 (9,20,10,'2026-03-11','REFUNDED'),(10,30,1000,'2026-01-01','PAID');
