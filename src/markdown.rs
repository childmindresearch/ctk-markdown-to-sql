/// Represents a node in a tree structure of markdown elements.
///
/// Each node contains text, a list of child nodes, and the depth level
/// in the markdown document where it was found.

static PARAGRAPH_DEPTH: usize = 99999;

pub struct TreeNode {
    pub text: String,
    pub children: Vec<TreeNode>,
    pub depth: usize,
}

impl TreeNode {
    /// Walks through lines of markdown text to build a tree structure,
    /// starting from the current node.
    ///
    /// # Arguments
    /// * `lines` - A mutable peekable iterator over lines of markdown text
    ///             and their depth in the document.
    fn walk<'a>(
        &mut self,
        lines: &mut std::iter::Peekable<impl Iterator<Item = (usize, &'a str)>>,
    ) {
        while let Some(&(depth, text)) = lines.peek() {
            if depth < self.depth {
                return;
            }
            if depth == self.depth && depth != PARAGRAPH_DEPTH {
                return;
            }
            lines.next();
            if depth == PARAGRAPH_DEPTH && self.depth == PARAGRAPH_DEPTH {
                self.text.push('\n');
                self.text += text.trim();
                return;
            }

            let mut node = TreeNode {
                text: text.trim_start_matches('#').trim().to_owned(),
                children: Vec::new(),
                depth,
            };
            node.walk(lines);
            self.children.push(node)
        }
    }
}

/// Responsible for converting a tree of `TreeNode`s into SQL commands.
///
/// Tracks the next available ID for SQL insertion and stores the generated SQL queries.
pub struct TreeToSql {
    pub next_id: usize,
    pub queries: Vec<String>,
}

impl TreeToSql {
    /// Writes the SQL table definition for storing the tree data.
    ///
    /// # Arguments
    /// * `table_name` - The name of the SQL table.
    ///
    /// # Returns
    /// Returns the SQL command to create the table.
    pub fn write_table_definition(&self, table_name: &str) -> String {
        return format!(
            "BEGIN TRANSACTION;
CREATE TABLE {} (
    id INTEGER NOT NULL,
    text VARCHAR(10000) NOT NULL,
    parent_id INTEGER,
    PRIMARY KEY (id),
    FOREIGN KEY (parent_id) REFERENCES {} (id)
);\n",
            table_name, table_name
        );
    }

    /// Converts a `TreeNode` into a series of SQL insert commands based on the tree structure.
    ///
    /// # Arguments
    /// * `tree` - The root `TreeNode` from which to start the SQL conversion.
    /// * `table_name` - The name of the table to insert data into.
    ///
    /// # Returns
    /// Returns a String containing all SQL insert commands for the tree.
    pub fn write_sql_insertions(&mut self, tree: TreeNode, table_name: &str) -> String {
        self._write_sql_insertions_loop(tree, table_name, None);
        return self.queries.join("\n");
    }

    /// Helper function to recursively generate SQL insert commands from a `TreeNode`.
    ///
    /// # Arguments
    /// * `tree` - The current `TreeNode`.
    /// * `table_name` - The name of the table to insert data into.
    /// * `parent_id` - Optional parent ID for the current node, `None` if it is the root.
    fn _write_sql_insertions_loop(
        &mut self,
        tree: TreeNode,
        table_name: &str,
        parent_id: Option<usize>,
    ) {
        let parent_id_string = match parent_id {
            Some(number) => number.to_string(),
            None => "NULL".into(),
        };
        let this_id = self.next_id;
        self.next_id += 1;
        self.queries.push(format!(
            "INSERT INTO {} VALUES({}, {}, {});",
            table_name, this_id, tree.text, parent_id_string
        ));
        for child in tree.children {
            self._write_sql_insertions_loop(child, table_name, Some(this_id));
        }
    }
}

/// Parses a markdown string into a tree structure of `TreeNode`s.
///
/// # Arguments
/// * `markdown` - The markdown content as a String.
///
/// # Returns
/// Returns the root `TreeNode` of the parsed markdown document.
pub fn parse_markdown_tree(markdown: &str, root_name: &str) -> TreeNode {
    let mut levels = markdown
        .split("\n")
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut leading_hashes = count_leading_characters(line, '#');
            if leading_hashes == 0 {
                leading_hashes = PARAGRAPH_DEPTH
            }
            return (leading_hashes, line);
        })
        .peekable();

    let mut root = TreeNode {
        text: String::from(root_name),
        children: Vec::new(),
        depth: 0,
    };
    root.walk(&mut levels);
    return root;
}

/// Counts the number of consecutive characters at the beginning of a string.
///
/// # Arguments
/// * `line` - The string slice to inspect.
/// * `character` - The character to count.
///
/// # Returns
/// Returns the count of consecutive characters at the start of the string.
fn count_leading_characters(line: &str, character: char) -> usize {
    return line.chars().take_while(|char| char == &character).count();
}

#[test]
fn test_count_leading_characters_none() {
    assert_eq!(count_leading_characters("Hello World", '!'), 0)
}

#[test]
fn test_count_leading_characters_some() {
    assert_eq!(count_leading_characters("##!#Hello World", '#'), 2)
}

#[test]
fn test_parse_markdown_tree() {
    let markdown = "# Header 1
## Header 2.1

text

## Header 2.2

text1
text2
";

    let tree = parse_markdown_tree(markdown, "root");

    assert_eq!(tree.text, "root", "root node is named root");
    assert_eq!(tree.children.len(), 1, "root node has one child");
    assert_eq!(
        tree.children[0].children.len(),
        2,
        "two second level headers exist"
    );
    assert_eq!(
        tree.children[0].children[1].children.len(),
        1,
        "header 2.2 has one child"
    );
    assert_eq!(
        tree.children[0].children[1].children[0].text, "text1\ntext2",
        "paragraphs are correctly concatenated"
    );
}

#[test]
fn test_write_table_definition() {
    let tree_to_sql = TreeToSql {
        next_id: 1,
        queries: vec![],
    };

    let table_definition = tree_to_sql.write_table_definition("templates");

    assert_eq!(
        table_definition,
        "BEGIN TRANSACTION;
CREATE TABLE templates (
    id INTEGER NOT NULL,
    text VARCHAR(10000) NOT NULL,
    parent_id INTEGER,
    PRIMARY KEY (id),
    FOREIGN KEY (parent_id) REFERENCES templates (id)
);\n"
    );
}

#[test]
fn test_write_sql_insertion() {
    let mut tree_to_sql = TreeToSql {
        next_id: 1,
        queries: vec![],
    };

    let tree = TreeNode {
        text: "root".to_string(),
        children: vec![TreeNode {
            text: "child".to_string(),
            children: vec![],
            depth: 1,
        }],
        depth: 0,
    };

    let sql = tree_to_sql.write_sql_insertions(tree, "templates");

    assert_eq!(
        sql,
        "INSERT INTO templates VALUES(1, root, NULL);
INSERT INTO templates VALUES(2, child, 1);"
    );
}
