use anyhow::{Result, bail, Context};
use std::{collections::HashMap, env, fs::File};
use csv::{StringRecord, WriterBuilder};

struct ColumnMapping<'t> {
    column: &'t str,
    title: &'t str,
    shorten: bool,
}

type SColumnMapping = ColumnMapping<'static>;

struct Filter<'t> {
    column: &'t str,
    value: &'t str,
}

type SFilter = Filter<'static>;

static COLUMN_MAPPINGS: &[ColumnMapping] = &[
    SColumnMapping {
        column: "objectname",
        title: "Game",
        shorten: false,
    },
    SColumnMapping {
        column: "minplayers",
        title: "Min\nPlayers",
        shorten: false,
    },
    SColumnMapping {
        column: "maxplayers",
        title: "Max\nPlayers",
        shorten: false,
    },
    SColumnMapping {
        column: "playingtime",
        title: "Playing-\ntime",
        shorten: false,
    },
    SColumnMapping {
        column: "minplaytime",
        title: "Min \nPlaytime",
        shorten: false,
    },
    SColumnMapping {
        column: "maxplaytime",
        title: "Max \nPlaytime",
        shorten: false,
    },
    SColumnMapping {
        column: "yearpublished",
        title: "Year",
        shorten: false,
    },
    SColumnMapping {
        column: "bggbestplayers",
        title: "Best Amount \nPlayers",
        shorten: false,
    },
    SColumnMapping {
        column: "bggrecagerange",
        title: "Age \nRange",
        shorten: false,
    },
    SColumnMapping {
        column: "itemtype",
        title: "Type",
        shorten: true,
    },
    SColumnMapping {
        column: "version_languages",
        title: "Language",
        shorten: false,
    },
    ];

static FILTERS: &[Filter] = &[
    SFilter {
        column: "comment",
        value: "Bei der Arbeit",
    },
    SFilter {
        column: "prevowned",
        value: "1",
    },
];

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 2 {
        bail!("No parameters provided. Please provide a CSV input and output file name.");
    }
    let input_file = &args[0];
    let output_file = &args[1];
    println!("Reading file {input_file}");
    let open_file = File::open(input_file).context("Failed to open file")?;
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(open_file);
    let mut records = rdr.records();
    let headers = records.next().expect("No lines in csv file").context("")?;
    
    let heading_to_index : HashMap<&str, usize> = headers.iter().enumerate().map(|(i, m)| (m,i)).collect();
    
    let mappings = init_lookup(&heading_to_index, COLUMN_MAPPINGS).context("Failed to initialize column mappings")?;
    let row_filter = create_filter_predicate(&heading_to_index, FILTERS);

    println!("Writing output to file {output_file}");
    let mut wtr = WriterBuilder::new().from_path(output_file)?;
    
    // write headers
    let headers : StringRecord = mappings.iter().map(|(_, mapping)| mapping.title).collect();
    wtr.write_record(&headers).context("Writing to output")?;


    for record in records.filter(row_filter) {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here.
        let record = record?;
        let out = collect_into_record(&mappings, &record);
        wtr.write_record(&out).context("Writing to output")?;
    }

    Ok(())
}

fn create_filter_predicate(heading_to_index : &HashMap<&str, usize>, filters : &[Filter]) -> impl FnMut(&Result<StringRecord, csv::Error>) -> bool { 
        let field_filters : Vec<(usize, &str)> = filters.iter().map(|filter| (heading_to_index[filter.column], filter.value)).collect();
        return move |record| {
            let Ok(record) = record else {
                // if we have an error, we accept it. The filter does not care.
                return true;
            };
            field_filters.iter().all(|(index, val)| &record[*index] == *val)
        }
}

fn collect_into_record(mappings: &Vec<(usize, &ColumnMapping<'_>)>, record: &StringRecord) -> StringRecord {
    let out : StringRecord = mappings.iter().map(|(index, mapping)| get_and_shorten(record, *index, *mapping)).collect();
    out
}

fn get_and_shorten<'a>(record : &'a StringRecord, index : usize, mapping : &ColumnMapping) -> &'a str {
    let field = &record[index];
    return if mapping.shorten {
        &field[..1]
    } else {
        field
    }
}

fn init_lookup<'col>(heading_to_index: &HashMap<&str, usize>, mappings : &'col[ColumnMapping<'col>]) -> Result<Vec<(usize, &'col ColumnMapping<'col>)>> {
    let mut results = Vec::new();
    for mapping in mappings {
        if let Some(index) = heading_to_index.get(mapping.column) {
            results.push((*index, mapping));
        } else {
            bail!("Column '{}' not found in CSV headers", mapping.column);
        }
    }
    Ok(results)
}

// fn print_type(val: &Value) {
//     let the_type = match val {
//         Value::Array(_) => "Array",
//         Value::Null => "null",
//         Value::Bool(_) => "bool",
//         Value::Number(_) => "number",
//         Value::String(_) => "string",
//         Value::Object(_) => "object",
//     };
//     println!("Type: {}", the_type);
// }
