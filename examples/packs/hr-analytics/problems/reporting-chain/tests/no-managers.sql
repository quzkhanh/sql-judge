INSERT INTO departments (id, name) VALUES
  (40, 'Support');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (401, 'Phong', 'phong@example.com', 40, NULL, 3000, '2021-03-01'),
  (402, 'Quang', 'quang@example.com', 40, NULL, 3100, '2021-03-01');
