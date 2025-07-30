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

Then `ctk-md-to-sql -i my_example_root test.md -o test.sql` will generate the SQL insertion. Note that there's currently a bug where the last comma before the final SELECT statement should be manually removed.
