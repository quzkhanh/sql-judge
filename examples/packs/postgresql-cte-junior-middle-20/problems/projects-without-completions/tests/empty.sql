-- Independent no-task fixture: existing project but no fact rows.
INSERT INTO users (id,name,is_active,joined_at) VALUES (999,'Empty Owner',TRUE,'2026-01-01');
INSERT INTO projects (id,name,owner_id,status) VALUES (999,'No Task Project',999,'ACTIVE');
