pub mod crap;
pub mod sifis;
pub mod skunk;
pub mod utility;

use crate::crap::crap;
use crate::sifis::{sifis_plain, sifis_quantized};
use crate::skunk::skunk_nosmells;
use crate::utility::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::*;

/// Struct with all the metrics computed for a single file
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Metrics {
    sifis_plain: f64,
    sifis_quantized: f64,
    crap: f64,
    skunk: f64,
    file: String,
}
/// This Function get the folder of the repo to analyzed and the path to the json obtained using grcov
/// It prints all the SIFIS, CRAP and SkunkScore values for all the files in the folders
/// the output will be print as follows:
/// FILE       | SIFIS PLAIN | SIFIS QUANTIZED | CRAP       | SKUNKSCORE
/// if the a file is not found in the json that files will be skipped
pub fn get_metrics_output(
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
) -> Result<(), SifisError> {
    println!(
        "{0: <20} | {1: <20} | {2: <20} | {3: <20} | {4: <20}",
        "FILE", "SIFIS PLAIN", "SIFIS QUANTIZED", "CRAP", "SKUNKSCORE"
    );
    for m in metrics {
        println!(
            "{0: <20} | {1: <20.3} | {2: <20.3} | {3: <20.3} | {4: <20.3}",
            m.file, m.sifis_plain, m.sifis_quantized, m.crap, m.skunk
        );
    }
    println!("FILES IGNORED: {}", files_ignored.len());
    Ok(())
}

/// This Function get the folder of the repo to analyzed and the path to the json obtained using grcov
/// if the a file is not found in the json that files will be skipped
/// It returns a tuple with a vector with all the metrics for a file and the comulative values and a vector with the list of all ignored files
pub fn get_metrics<A: AsRef<Path> + Copy, B: AsRef<Path> + Copy>(
    files_path: A,
    json_path: B,
    metric: COMPLEXITY,
) -> Result<(Vec<Metrics>, Vec<String>), SifisError> {
    let vec = match read_files(files_path.as_ref()) {
        Ok(vec) => vec,
        Err(_err) => {
            return Err(SifisError::WrongFile(
                files_path.as_ref().display().to_string(),
            ))
        }
    };
    let mut files_ignored: Vec<String> = Vec::<String>::new();
    let mut res = Vec::<Metrics>::new();
    let file = match fs::read_to_string(json_path) {
        Ok(file) => file,
        Err(_err) => {
            return Err(SifisError::WrongFile(
                json_path.as_ref().display().to_string(),
            ))
        }
    };
    let covs = read_json(file, files_path.as_ref().to_str().unwrap())?;
    for path in vec {
        let p = Path::new(&path);
        let file = p.file_name().unwrap().to_str().unwrap().to_string();
        let arr = match covs.get(&path) {
            Some(arr) => arr.to_vec(),
            None => {
                files_ignored.push(file);
                continue;
            }
        };
        let root = get_root(p)?;
        let sifis_plain = sifis_plain(&root, &arr, metric)?;
        let sifis_quantized = sifis_quantized(&root, &arr, metric)?;
        let crap = crap(&root, &arr, metric)?;
        let skunk = skunk_nosmells(&root, &arr, metric)?;
        res.push(Metrics {
            sifis_plain,
            sifis_quantized,
            crap,
            skunk,
            file,
        });
    }
    let (avg, min, max) = get_cumulative_values(&res);
    res.push(avg);
    res.push(min);
    res.push(max);
    Ok((res, files_ignored))
}

///Prints the reulst of the get_metric function in a csv file
/// the structure is the following :
/// FILE,SIFIS PLAIN,SIFIS QUANTAZED,CRAP,SKUNK
pub fn print_metrics_to_csv<A: AsRef<Path> + Copy>(
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
    csv_path: A,
) -> Result<(), SifisError> {
    export_to_csv(csv_path.as_ref(), metrics, files_ignored)
}

pub fn print_metrics_to_json<A: AsRef<Path> + Copy>(
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
    json_output: A,
    project_folder: A,
) -> Result<(), SifisError> {
    export_to_json(
        project_folder.as_ref(),
        json_output.as_ref(),
        metrics,
        files_ignored,
    )
}
