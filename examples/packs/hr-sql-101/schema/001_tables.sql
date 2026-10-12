CREATE TABLE departments (
  id integer PRIMARY KEY,
  name text NOT NULL
);
CREATE TABLE employees (
  id integer PRIMARY KEY,
  name text NOT NULL,
  department_id integer NOT NULL REFERENCES departments(id),
  salary numeric(12, 2)
);
