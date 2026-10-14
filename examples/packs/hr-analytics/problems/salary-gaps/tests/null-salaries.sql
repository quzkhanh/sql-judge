INSERT INTO departments (id, name) VALUES
  (20, 'Legal'),
  (30, 'Sales');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (201, 'Em', 'em@example.com', 20, NULL, 4000, '2020-01-01'),
  (202, 'Giang', 'giang@example.com', 20, NULL, NULL, '2020-02-01'),
  (203, 'Ha', 'ha@example.com', 20, NULL, 4500.5, '2020-03-01'),
  (301, 'Hieu', 'hieu@example.com', 30, NULL, NULL, '2021-01-01');
