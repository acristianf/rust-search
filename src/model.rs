use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use xml::reader::EventReader;
use xml::reader::XmlEvent;

use serde::{Deserialize, Serialize};

use crate::lexer::*;

// TYPES DEFINITIONS
pub type DocFreq = HashMap<String, usize>;
pub type TermFreq = HashMap<String, usize>;
pub type TermFreqPerDoc = HashMap<PathBuf, (usize, TermFreq)>;

#[derive(Default, Deserialize, Serialize)]
pub struct Model {
    pub df: DocFreq,
    pub tfpd: TermFreqPerDoc,
}

pub fn compute_tf(t: &str, d: &(usize, TermFreq)) -> f32 {
    let sum: usize = d.0;
    *d.1.get(t).unwrap_or(&0) as f32 / sum as f32
}

pub fn compute_idf(t: &str, n: usize, df: &DocFreq) -> f32 {
    let n = n as f32;
    let m: f32 = df.get(t).cloned().unwrap_or(0) as f32;
    f32::log10(n / (1.0 + m))
}

pub fn read_xml_file(file_path: &Path) -> io::Result<String> {
    let file = File::open(file_path)?;
    let file = BufReader::new(file);
    let reader = EventReader::new(file);
    let mut contents = String::new();
    for result in reader.into_iter() {
        if let XmlEvent::Characters(text) = result.expect("TODO") {
            contents.push_str(&text);
            contents.push_str(" "); // Pad last word
        }
    }
    Ok(contents)
}

pub fn add_folder_to_model(dir_path: &Path, model: &mut Model) {
    let dir = fs::read_dir(dir_path).unwrap();
    let ext_xhtml = std::ffi::OsStr::new("xhtml");
    for file in dir {
        let file_path = file.unwrap().path();
        if file_path.is_dir() {
            add_folder_to_model(&file_path, model);
        }

        let ext = file_path
            .extension()
            .unwrap_or_else(|| std::ffi::OsStr::new(""));

        if ext_xhtml == ext {
            let content = read_xml_file(&file_path)
                .unwrap()
                .chars()
                .collect::<Vec<_>>();

            println!("Indexing... {file_path:?}");

            let mut tf = TermFreq::new();

            for term in Lexer::new(&content) {
                if let Some(count) = tf.get_mut(&term) {
                    *count += 1;
                } else {
                    tf.insert(term, 1);
                }
            }

            let mut term_sum: usize = 0;
            for t in tf.keys() {
                if let Some(freq) = model.df.get_mut(t) {
                    *freq += 1;
                } else {
                    model.df.insert(t.to_string(), 1);
                };
                term_sum += 1;
            }

            let mut stats = tf.iter().collect::<Vec<_>>();
            stats.sort_by_key(|(_, f)| *f);
            stats.reverse();

            model.tfpd.insert(file_path, (term_sum, tf));
        }
    }
}

pub fn save_model_as_json(index_file: &str, model: &Model) -> std::io::Result<()> {
    let index_path = index_file;
    println!("Saving {index_path}...");
    let index_file = File::create(index_path)?;
    let index_file = BufWriter::new(index_file);
    serde_json::to_writer(index_file, model).map_err(|err| {
        eprintln!("Error: couldn't save model as JSON; {err}");
    }).unwrap();
    Ok(())
}

pub fn load_index(index_path: &Path) -> Result<Model, ()> {
    let index_file = File::open(index_path).unwrap();
    let mut index_reader = io::BufReader::new(index_file);
    let model: Model =
        serde_json::from_reader(&mut index_reader).expect("serde works fine");
    Ok(model)
}

pub fn search_query<'a>(
    query: &'a Vec<char>,
    model: &'a Model,
) -> Vec<(&'a Path, f32)> {
    let mut tf_idf = 0.0;
    let mut rank = Vec::<(&Path, f32)>::new();
    for (file, d) in model.tfpd.iter() {
        for token in Lexer::new(&query) {
            tf_idf += compute_tf(&token, &d) * compute_idf(&token, model.tfpd.len(), &model.df);
        }
        if tf_idf > 0.0 {
            rank.push((file, tf_idf));
        }
        tf_idf = 0.0;
    }
    rank.sort_by(|(_, rank1), (_, rank2)| rank2.partial_cmp(rank1).unwrap());
    rank
}
