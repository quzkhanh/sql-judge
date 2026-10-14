INSERT INTO departments (id, name) VALUES
  (30, 'Support'),
  (40, 'Marketing');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (301, 'Ha', 'ha@example.com', 30, NULL, NULL, '2021-03-01'),
  (302, 'Hieu', 'hieu@example.com', 30, NULL, 3000, '2021-03-01'),
  (303, 'Khoa', 'khoa@example.com', 30, NULL, 2000, '2021-03-01'),
  (401, 'Long', 'long@example.com', 40, NULL, NULL, '2021-03-01'),
  (402, 'My', 'my@example.com', 40, NULL, NULL, '2021-03-01');
