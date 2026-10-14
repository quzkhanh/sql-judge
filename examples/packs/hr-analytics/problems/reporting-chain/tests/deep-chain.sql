INSERT INTO departments (id, name) VALUES
  (10, 'Research');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (101, 'An', 'an@example.com', 10, NULL, 9000, '2021-03-01'),
  (102, 'Binh', 'binh@example.com', 10, 101, 8000, '2021-03-01'),
  (103, 'Chi', 'chi@example.com', 10, 102, 7000, '2021-03-01'),
  (104, 'Dung', 'dung@example.com', 10, 103, 6000, '2021-03-01'),
  (105, 'Em', 'em@example.com', 10, 104, 5000, '2021-03-01'),
  (106, 'Giang', 'giang@example.com', 10, 105, 4000, '2021-03-01');
