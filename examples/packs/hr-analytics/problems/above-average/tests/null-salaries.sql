INSERT INTO departments (id, name) VALUES
  (30, 'Support'),
  (40, 'Marketing');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (301, 'Khanh', 'khanh@example.com', 30, NULL, 400, '2021-03-01'),
  (302, 'Lan', 'lan@example.com', 30, NULL, NULL, '2021-03-01'),
  (303, 'Mai', 'mai@example.com', 30, NULL, 100, '2021-03-01'),
  (304, 'Nam', 'nam@example.com', 30, NULL, 100, '2021-03-01'),
  (401, 'Oanh', 'oanh@example.com', 40, NULL, NULL, '2021-03-01'),
  (402, 'Phuc', 'phuc@example.com', 40, NULL, NULL, '2021-03-01');
