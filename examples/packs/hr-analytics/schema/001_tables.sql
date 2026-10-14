CREATE TABLE departments (
  id integer PRIMARY KEY,
  name text NOT NULL
);
CREATE TABLE employees (
  id integer PRIMARY KEY,
  name text NOT NULL,
  email text NOT NULL,
  department_id integer NOT NULL REFERENCES departments(id),
  manager_id integer REFERENCES employees(id),
  salary numeric,
  hired_on date NOT NULL
);
