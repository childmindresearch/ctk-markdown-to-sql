/// Entry point of the program.
/// Parses command line arguments to get a markdown file name,
/// reads the file, converts the markdown into a SQL query to insert the
/// data.
mod markdown;
use clap::{Arg, ArgAction, Command};
use itertools::Itertools;

fn main() -> Result<(), std::io::Error> {
    let args = parse_args();
    let root = markdown::TreeNode {
        text: "root".to_string(),
        depth: 0,
        children: args
            .input_files
            .iter()
            .map(|file| markdown::parse_markdown_tree(&read_file(&file.file), &file.name))
            .collect(),
    };

    let mut tree_to_sql = markdown::TreeToSql {
        next_id: 1,
        queries: vec![],
    };

    let table_name = "templates";
    let sql_query =
        format!("DROP TABLE IF EXISTS {};", table_name) +
        &tree_to_sql.write_table_definition(table_name) + 
        &tree_to_sql.write_sql_insertions(root, table_name) + 
        &tree_to_sql.convert_id_to_auto_increment(table_name) +
        "\nCOMMIT;";

    let write_result = std::fs::write(args.output_file, &sql_query);
    if write_result.is_err() {
        eprintln!("Could not write to output file.",);
    }
    return write_result;
}

fn read_file(filename: &str) -> String {
    return std::fs::read_to_string(filename)
        .expect("Input files should be readable plaintext files.");
}

struct InputFileArg {
    name: String,
    file: String,
}

struct Arguments {
    input_files: Vec<InputFileArg>,
    output_file: String,
}

fn parse_args() -> Arguments {
    let parser = Command::new("Markdown to SQL")
        .version("0.1.0")
        .author("Reinder Vos de Wael <reinder.vosdewael@childmind.org>")
        .about("Converts markdown files to SQL tables.")
        .arg(
            Arg::new("input_file")
                .action(ArgAction::Append)
                .required(true)
                .short('i')
                .long("input")
                .value_names(&["name", "file"])
                .number_of_values(2)
                .help("Takes two arguments: the root name of the tree in SQL and the filepath."),
        )
        .arg(
            Arg::new("output_file")
                .short('o')
                .long("output")
                .value_name("file")
                .required(true)
                .help("The file to write the SQL output to."),
        )
        .get_matches();

    let output_file = parser
        .get_one::<String>("output_file")
        .expect("No output file provided; this should have been caught by the parser.");

    let input_values = parser
        .get_many::<String>("input_file")
        .expect("Error in input arguments; this should have been caught by the parser.");
    let input_files: Vec<InputFileArg> = input_values
        .into_iter()
        .chunks(2)
        .into_iter()
        .map(|mut chunk| InputFileArg {
            name: chunk
                .next()
                .expect("Unexpected error in input argument parsing; this should have been caught by the parser.")
                .to_string(),
            file: chunk
                .next()
                .expect("Unexpected error in input argument parsing;his should have been caught by the parser.")
                .to_string(),
        })
        .collect();
    return Arguments {
        input_files,
        output_file: output_file.to_owned(),
    };
}
