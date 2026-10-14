INSERT INTO departments (id, name) VALUES
  (10, 'Research');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (103, 'Chi', 'chi@example.com', 10, NULL, 7000, '2023-01-02'),
  (101, 'An', 'an@example.com', 10, NULL, 5000, '2023-01-02'),
  (102, 'Binh', 'binh@example.com', 10, NULL, 6000, '2023-01-02'),
  (104, 'Dung', 'dung@example.com', 10, NULL, 6000, '2023-02-01');
