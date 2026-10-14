INSERT INTO departments (id, name) VALUES
  (1, 'Engineering'),
  (2, 'Finance');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (1, 'Linh', 'linh@example.com', 1, NULL, 135000, '2021-03-01'),
  (2, 'Minh', 'minh@example.com', 1, NULL, 100000, '2021-03-01'),
  (3, 'Hoa', 'hoa@example.com', 1, NULL, 90000, '2021-03-01'),
  (4, 'Quan', 'quan@example.com', 2, NULL, 98000, '2021-03-01'),
  (5, 'Trang', 'trang@example.com', 2, NULL, 72000, '2021-03-01');
