#![feature(read_initializer)]

extern crate tar;

use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, stdout};
use std::marker::PhantomData;
use std::ops::Deref;
use std::option::Option::Some;

use clap::Clap;
use csv::Writer;
use flate2::read::GzDecoder;
use quick_xml::events::{BytesStart, BytesText};
use quick_xml::events::Event::*;
use quick_xml::Reader;
use serde::Serialize;
use serde_json;
use tar::{Archive, Entry};

const BUF_CAPACITY: usize = 2097152;

#[derive(Clap)]
#[clap(version = "0.1", author = "James Schinner")]
struct Opts {
    #[clap(short, long)]
    file: String,

    #[clap(short, long)]
    destination: String,
}


trait Gather {
    fn new() -> Self where Self: Sized;
    fn gather_attr(&mut self, path: &Vec<&[u8]>, bytes_start: BytesStart<'_>);
    fn gather_text(&mut self, path: &Vec<&[u8]>, bytes_start: BytesText<'_>);
}

trait ReAddress {
    unsafe fn re_address(&mut self, original_pnt: *const u8, new_pnt: *const u8);
}


struct FileParser<'a, D: Gather + ReAddress> {
    buf_depth: i32,
    depth: i32,
    buf: Vec<u8>,
    reader: Reader<BufReader<Entry<'a, GzDecoder<File>>>>,
    path: Vec<&'a [u8]>,
    data: Option<D>,
}


impl<'a, D: Gather + ReAddress> FileParser<'a, D> {
    fn new(buf_depth: i32, file: BufReader<Entry<'a, GzDecoder<File>>>) -> FileParser<'a, D> {
        let mut reader = Reader::from_reader(file);
        reader.expand_empty_elements(true);
        reader.trim_text(true);
        FileParser {
            buf_depth,
            depth: 0,
            buf: Vec::with_capacity(BUF_CAPACITY),
            reader,
            path: vec![],
            data: None,
        }
    }


    fn next(&mut self) -> Option<&D> {
        assert_eq!(self.buf.capacity(), BUF_CAPACITY,
                   "Read buffer has increased in size from {} to {}. \
                   This means the buffer has been reallocated and pointers \
                   are now invalid. Try recompiling this program with a BUF_CAPACITY \
                   of at least {}", BUF_CAPACITY, self.buf.capacity(), self.buf.capacity()
        );
        self.buf.clear();
        self.path.clear();
        self.data = Some(D::new());
        let original_pnt = self.buf.as_ptr();
        loop {
            match self.reader.read_event(&mut self.buf) {
                Ok(Start(bytes_start)) => {
                    self.depth += 1;
                    if self.depth >= self.buf_depth {
                        unsafe {
                            self.path.push(
                                std::slice::from_raw_parts(
                                    bytes_start.name().deref().as_ptr(),
                                    bytes_start.name().deref().len(),
                                )
                            )
                        }
                        if let Some(data) = &mut self.data {
                            data.gather_attr(&self.path, bytes_start);
                        }
                    }
                }
                Ok(Text(bytes_text)) => {
                    if let Some(data) = &mut self.data {
                        data.gather_text(&self.path, bytes_text);
                    }
                }

                Ok(End(_)) => {
                    self.depth -= 1;
                    self.path.pop();
                    if self.depth == self.buf_depth - 1 {
                        break;
                    }
                }
                Ok(Eof) => {
                    self.data = None;
                    break;
                }
                _ => {}
            }
            if self.depth < self.buf_depth {
                self.buf.clear();
                self.path.clear();
            }

            if self.buf.as_ptr() != original_pnt {
                if let Some(data) = &mut self.data {
                    unsafe {data.re_address(original_pnt, self.buf.as_ptr())}
                }
            }
        }
        self.data.as_ref()
    }
}

unsafe fn get_attribute(bytes_start: &BytesStart, attribute: &[u8]) -> Option<&'static str> {
    // Return an attribute from the BytesStart's underlying buffer.
    // Note the returned references will have a static life time! This is not safe code!!!
    match bytes_start.attributes()
        .filter_map(|attr| {
            let attr = attr.unwrap();
            if attr.key == attribute {
                Some(attr.value)
            } else {
                None
            }
        }).next() {
        Some(cow) => {
            let pnt = std::slice::from_raw_parts(cow.as_ptr(), cow.len());
            let str = std::str::from_utf8_unchecked(pnt);
            Some(str)
        }
        _ => { None }
    }
}

unsafe fn get_text(bytes_text: &BytesText) -> Option<&'static str> {
    let pnt = std::slice::from_raw_parts(
        bytes_text.escaped().as_ptr()
        , bytes_text.escaped().len(),
    );
    let str = std::str::from_utf8_unchecked(pnt);
    Some(str)
}

// fn get_text(bytes_text: BytesText) -> Option<&'a >

#[derive(Debug, Serialize)]
struct Experiment<'a> {
    srx: Option<&'a str>,
    srp: Option<&'a str>,
    srs: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct Study<'a> {
    srp: Option<&'a str>,
    alias: Option<&'a str>,
    title: Option<&'a str>,
    ab: Option<&'a str>,

}

impl ReAddress for Experiment{
    unsafe fn re_address(&mut self, original_pnt: *const u8, new_pnt: *const u8) {
        unimplemented!()
    }
}

const EXPERIMENT: &[u8] = b"EXPERIMENT";
const STUDY_REF: &[u8] = b"STUDY_REF";
const DESIGN: &[u8] = b"DESIGN";
const SAMPLE_DESCRIPTOR: &[u8] = b"SAMPLE_DESCRIPTOR";

impl<'a> Gather for Experiment<'a> {
    fn new() -> Self {
        Experiment { srx: None, srp: None, srs: None }
    }

    fn gather_attr(&mut self, path: &Vec<&[u8]>, bytes_start: BytesStart<'_>) {
        unsafe {
            if *path == [EXPERIMENT] {
                self.srx = get_attribute(&bytes_start, b"accession")
            } else if *path == [EXPERIMENT, STUDY_REF] {
                self.srp = get_attribute(&bytes_start, b"accession")
            } else if *path == [EXPERIMENT, DESIGN, SAMPLE_DESCRIPTOR] {
                //ExperimentSet/EXPERIMENT/DESIGN/SAMPLE_DESCRIPTOR
                self.srs = get_attribute(&bytes_start, b"accession")
            }
        }
    }

    fn gather_text(&mut self, _: &Vec<&[u8]>, _: BytesText<'_>) {}
}

const STUDY: &[u8] = b"STUDY";
const DESCRIPTOR: &[u8] = b"DESCRIPTOR";
const STUDY_TITLE: &[u8] = b"STUDY_TITLE";
const STUDY_ABSTRACT: &[u8] = b"STUDY_ABSTRACT";

impl<'a> Gather for Study<'a> {
    fn new() -> Self {
        Study {
            srp: None,
            alias: None,
            title: None,
            ab: None,
        }
    }

    fn gather_attr(&mut self, path: &Vec<&[u8]>, bytes_start: BytesStart<'_>) {
        unsafe {
            if *path == [STUDY] {
                self.srp = get_attribute(&bytes_start, b"accession");
                self.alias = get_attribute(&bytes_start, b"alias");
            }
        }
    }

    fn gather_text(&mut self, path: &Vec<&[u8]>, bytes_text: BytesText<'_>) {
        unsafe {
            // /STUDY/DESCRIPTOR/STUDY_TITLE
            if *path == [STUDY, DESCRIPTOR, STUDY_TITLE] {
                self.title = get_text(&bytes_text);
            } else if *path == [STUDY, DESCRIPTOR, STUDY_ABSTRACT] {
                self.ab = get_text(&bytes_text);
            }
        }
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    let handle = stdout();
    let mut writer = handle.lock();
    let tar_gz = File::open(opts.file).unwrap();
    let tar = GzDecoder::new(tar_gz);
    let mut a = Archive::new(tar);
    let mut experiment_writer = Writer::from_path(
        format!("{}/experiments.csv", opts.destination)
    )
        .expect("Unable to create experiments.csv file");
    let mut study_writer = Writer::from_path(
        format!("{}/studies.csv", opts.destination)
    )
        .expect("Unable to create studies.csv file");
    for res in a.entries().unwrap() {
        let file = res.unwrap();
        let doc_path = &file.header().path().unwrap();
        let doc_str = doc_path.to_str().unwrap();

        let result = if doc_str.contains("experiment.xml") {
            let mut buf_reader = BufReader::with_capacity(BUF_CAPACITY, file);
            let mut parser = FileParser::<Experiment>::new(2, buf_reader);
            while let Some(data) = parser.next() {
                experiment_writer.serialize(data);
            }
            experiment_writer.flush();
            Ok(())
        } else if doc_str.contains("study.xml") {
            let mut buf_reader = BufReader::new(file);
            let mut parser = FileParser::<Study>::new(2, buf_reader);
            while let Some(data) = parser.next() {
                study_writer.serialize(data);
            }
            study_writer.flush();
            Ok(())
        } else { Err(()) };
    }
}


