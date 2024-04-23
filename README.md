# ctk-markdown-to-sql

This repository contains a Rust application that converts markdown files into SQL commands. The application parses markdown files into a tree structure of elements, then generates SQL commands to insert the parsed data into a database. Specifically, given a Markdown file as follows:

```markdown
# Header 1

## Header 2.1

text

## Header 2.2

text1
text2
```

Then `ctk-md-to-sql -i my_example_root test.md -o test.sql` will generate the following:

```sql
BEGIN TRANSACTION;
CREATE TABLE templates (
id INTEGER NOT NULL,
text VARCHAR(10000) NOT NULL,
parent_id INTEGER,
PRIMARY KEY (id),
FOREIGN KEY (parent_id) REFERENCES templates (id)
);
INSERT INTO templates VALUES(1, root, NULL);
INSERT INTO templates VALUES(2, my_example_root, 1);
INSERT INTO templates VALUES(3, Header 1, 2);
INSERT INTO templates VALUES(4, Header 2.1, 3);
INSERT INTO templates VALUES(5, text, 4);
INSERT INTO templates VALUES(6, Header 2.2, 3);
INSERT INTO templates VALUES(7, text1
text2, 6);
COMMIT;
```
