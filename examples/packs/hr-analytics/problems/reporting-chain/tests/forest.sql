INSERT INTO departments (id, name) VALUES
  (20, 'Legal'),
  (30, 'Sales');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (201, 'Ha', 'ha@example.com', 20, NULL, 6000, '2021-03-01'),
  (202, 'Hieu', 'hieu@example.com', 20, 201, 5000, '2021-03-01'),
  (203, 'Khoa', 'khoa@example.com', 20, 201, 5000, '2021-03-01'),
  (301, 'Long', 'long@example.com', 30, NULL, 4000, '2021-03-01'),
  (302, 'My', 'my@example.com', 30, 301, 3000, '2021-03-01'),
  (303, 'Nga', 'nga@example.com', 30, 302, NULL, '2021-03-01'),
  (304, 'Oanh', 'oanh@example.com', 30, NULL, 2500, '2021-03-01');
