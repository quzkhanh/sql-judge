INSERT INTO departments (id, name) VALUES
  (1, 'Engineering'),
  (2, 'Finance');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (1, 'Linh', 'linh@example.com', 1, NULL, 120000, '2021-01-04'),
  (2, 'Minh', 'minh@example.com', 1, NULL, 126000, '2022-03-01'),
  (3, 'Hoa', 'hoa@example.com', 1, NULL, 118500, '2023-06-12'),
  (4, 'Quan', 'quan@example.com', 2, NULL, 98000, '2022-08-15'),
  (5, 'Trang', 'trang@example.com', 2, NULL, 101000, '2024-02-01');
