INSERT INTO departments (id, name) VALUES
  (10, 'Research');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (101, 'An', 'an@example.com', 10, NULL, 1, '2021-12-31'),
  (102, 'Binh', 'binh@example.com', 10, NULL, 2, '2022-01-01'),
  (103, 'Chi', 'chi@example.com', 10, NULL, 3, '2022-12-31'),
  (104, 'Dung', 'dung@example.com', 10, NULL, 4, '2020-02-29');
