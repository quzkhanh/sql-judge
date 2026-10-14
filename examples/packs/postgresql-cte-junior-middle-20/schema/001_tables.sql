-- PostgreSQL 15+, shared schema; each fixture is independently loaded.
CREATE TABLE users (
    id integer PRIMARY KEY,
    name text NOT NULL,
    is_active boolean NOT NULL,
    joined_at date NOT NULL
);
CREATE TABLE projects (
    id integer PRIMARY KEY,
    name text NOT NULL,
    owner_id integer REFERENCES users(id),
    status text NOT NULL CHECK (status IN ('ACTIVE','ARCHIVED'))
);
CREATE TABLE tasks (
    id integer PRIMARY KEY,
    project_id integer NOT NULL REFERENCES projects(id),
    parent_task_id integer REFERENCES tasks(id),
    title text NOT NULL,
    status text CHECK (status IN ('AVAILABLE','IN_PROGRESS','DONE')),
    priority integer NOT NULL,
    created_at date NOT NULL,
    completed_at date,
    estimated_hours numeric(10,2),
    capacity integer NOT NULL CHECK (capacity > 0)
);
CREATE TABLE assignments (
    id integer PRIMARY KEY,
    task_id integer NOT NULL REFERENCES tasks(id),
    user_id integer NOT NULL REFERENCES users(id),
    status text NOT NULL CHECK (status IN ('ACTIVE','DONE','CANCELLED')),
    assigned_at date NOT NULL
);
CREATE TABLE work_logs (
    id integer PRIMARY KEY,
    task_id integer NOT NULL REFERENCES tasks(id),
    user_id integer NOT NULL REFERENCES users(id),
    logged_on date NOT NULL,
    hours numeric(10,2)
);
CREATE TABLE imports (
    id integer PRIMARY KEY,
    project_id integer NOT NULL REFERENCES projects(id),
    started_at date NOT NULL,
    status text NOT NULL
);
CREATE TABLE import_staging (
    id integer PRIMARY KEY,
    import_id integer NOT NULL REFERENCES imports(id),
    external_key text,
    user_id integer REFERENCES users(id),
    source_row_number integer NOT NULL,
    payload_status text
);
CREATE TABLE payments (
    id integer PRIMARY KEY,
    project_id integer NOT NULL REFERENCES projects(id),
    amount numeric(12,2),
    paid_at date NOT NULL,
    status text NOT NULL CHECK (status IN ('PAID','REFUNDED','PENDING'))
);
