INSERT INTO departments (id, name) VALUES
  (10, 'Research'),
  (20, 'Legal');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (101, 'An', 'an@example.com', 10, NULL, 9000, '2021-03-01'),
  (102, 'Binh', 'binh@example.com', 10, NULL, 9000, '2021-03-01'),
  (103, 'Chi', 'chi@example.com', 10, NULL, 8000, '2021-03-01'),
  (201, 'Dung', 'dung@example.com', 20, NULL, 5000, '2021-03-01'),
  (202, 'Em', 'em@example.com', 20, NULL, 5000, '2021-03-01'),
  (203, 'Giang', 'giang@example.com', 20, NULL, 5000, '2021-03-01');
