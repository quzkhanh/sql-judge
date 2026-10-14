INSERT INTO departments (id, name) VALUES
  (40, 'Support'),
  (50, 'Marketing'),
  (60, 'Finance');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (401, 'Nga', 'nga@example.com', 40, NULL, NULL, '2021-03-01'),
  (402, 'Oanh', 'oanh@example.com', 40, NULL, 8000, '2021-03-01'),
  (403, 'Phong', 'phong@example.com', 40, NULL, NULL, '2021-03-01'),
  (501, 'Quang', 'quang@example.com', 50, NULL, NULL, '2021-03-01'),
  (502, 'Son', 'son@example.com', 50, NULL, 6500, '2021-03-01'),
  (503, 'Tam', 'tam@example.com', 50, NULL, 5500, '2021-03-01'),
  (601, 'Uyen', 'uyen@example.com', 60, NULL, NULL, '2021-03-01');
