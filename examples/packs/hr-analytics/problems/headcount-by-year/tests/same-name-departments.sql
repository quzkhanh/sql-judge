INSERT INTO departments (id, name) VALUES
  (20, 'Operations'),
  (21, 'Operations'),
  (22, 'Empty Unit');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (201, 'Em', 'em@example.com', 20, NULL, 1, '2024-03-01'),
  (202, 'Giang', 'giang@example.com', 21, NULL, 2, '2024-07-15'),
  (203, 'Ha', 'ha@example.com', 21, NULL, NULL, '2024-11-30');
