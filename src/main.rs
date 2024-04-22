/// Entry point of the program.
/// Parses command line arguments to get a markdown file name,
/// reads the file, converts the markdown into a SQL query to insert the
/// data.
mod markdown;
use clap::{Arg, ArgAction, Command};
use itertools::Itertools;

fn main() {
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
    let table_definition = tree_to_sql.write_table_definition(table_name);
    let sql_query = table_definition + &tree_to_sql.write_sql_insertions(root, table_name);

    std::fs::write(args.output_file, &sql_query).unwrap_or_else(|err| {
        eprintln!("Could not write file: {}. Dumping to stdout.", err);
        println!("{}", &sql_query);
    });
}

fn read_file(filename: &str) -> String {
    return std::fs::read_to_string(filename)
        .unwrap_or_else(|err| panic!("Could not read Markdown file: {}", err));
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

    let output_file = parser.get_one::<String>("output_file").unwrap_or_else(|| {
        unreachable!("No output file provided. This should have been caught by the parser.");
    });

    if let Some(values) = parser.get_many::<String>("input_file") {
        let input_files: Vec<InputFileArg> = values
            .into_iter()
            .chunks(2)
            .into_iter()
            .map(|mut chunk| InputFileArg {
                name: chunk.next().unwrap().to_string(),
                file: chunk.next().unwrap().to_string(),
            })
            .collect();
        return Arguments {
            input_files,
            output_file: output_file.to_owned(),
        };
    }
    unreachable!("Error in input arguments. This should have been caught by the parser.")
}
