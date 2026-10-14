# HR Analytics

Sample `sqljudge.pack/v1` pack for SQL Judge. PostgreSQL 15+, eight problems over one
synthetic HR schema (`departments`, `employees`). Every fixture is self-contained and is
loaded into a freshly created copy of the schema; names and emails are synthetic.

| #   | Problem                               | Difficulty | Topics                          | Fixtures |
| --- | ------------------------------------- | ---------- | ------------------------------- | -------- |
| 1   | Employees above department average    | easy       | subqueries, aggregation         | 4        |
| 2   | Second highest salary per department  | medium     | window-functions, ranking       | 4        |
| 3   | Managers and their reporting chain    | hard       | recursive-cte                   | 4        |
| 4   | Duplicate employee emails             | easy       | aggregation                     | 4        |
| 5   | Department headcount by year          | easy       | aggregation, dates              | 4        |
| 6   | Top earners per department            | medium     | window-functions                | 4        |
| 7   | Departments without employees         | easy       | joins                           | 3        |
| 8   | Salary gap to the previous hire       | medium     | window-functions                | 4        |

Each problem has one public `basic` fixture (shown as the example in the app through
`catalog.sample_tables` and `catalog.sample_rows`) and hidden adversarial fixtures for ties,
`NULL` salaries and empty inputs. Hidden fixtures only control UI visibility; they are not
secret on an offline desktop.

Structural check (no SQL executed):

```bash
python3 scripts/validate_pack.py examples/packs/hr-analytics
```

The importer additionally runs every reference query against every fixture in a disposable
managed PostgreSQL environment before installing. The distributable archive
`examples/hr-analytics.sqljudge.zip` contains these files at the ZIP root.
