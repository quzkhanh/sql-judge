INSERT INTO departments (id, name) VALUES
  (10, 'Research'),
  (20, 'Legal'),
  (30, 'Sales');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (101, 'An', 'an@example.com', 10, NULL, 9000, '2021-03-01'),
  (102, 'Binh', 'binh@example.com', 10, NULL, 9000, '2021-03-01'),
  (103, 'Chi', 'chi@example.com', 10, NULL, 9000, '2021-03-01'),
  (104, 'Dung', 'dung@example.com', 10, NULL, 7000, '2021-03-01'),
  (105, 'Em', 'em@example.com', 10, NULL, 7000, '2021-03-01'),
  (106, 'Giang', 'giang@example.com', 10, NULL, 5000, '2021-03-01'),
  (201, 'Ha', 'ha@example.com', 20, NULL, 6000, '2021-03-01'),
  (202, 'Hieu', 'hieu@example.com', 20, NULL, 6000, '2021-03-01'),
  (301, 'Khoa', 'khoa@example.com', 30, NULL, 4000, '2021-03-01'),
  (302, 'Long', 'long@example.com', 30, NULL, 3000, '2021-03-01'),
  (303, 'My', 'my@example.com', 30, NULL, 3000, '2021-03-01');
