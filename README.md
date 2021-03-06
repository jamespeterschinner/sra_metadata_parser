# sra_metadata_parser

This project aims to create a simple method for researchers to extract the relevant
data for their needs from the SRA metadata dumps found here: 

ftp://ftp.ncbi.nlm.nih.gov/sra/reports/Metadata/

Thereby giving individuals a tool which can ensure they are using the most recent dataset.

The metadata dumps (.tar.gz files) contain millions of directory's each with a set of xml files 
that may include:

- *.experiment.xml
- *.run.xml
- *.sample.xml
- *.study.xml
- *.submission.xml

The idea is to specify which data is of interest within each xml file (using an XPath like strategy)
and the program will generate a .csv file (for now) for each corresponding xml document type 
(experiment.xml, run.xml ...). Each row in the csv file pertains to one group of associated data that
has been specified by the user (with in the compiled program).

At the time if writing (21/12/2020) this example program parses the entire SRA metadata dump under 15min 
on a 2.50Ghz i7 processor using ~3mb of RAM under 15min.

### Aims continued

I intend to develop this project into a SRA metadata dump -> SQL database program. After which 
I will split of the lower level functionality into another package which will expose an api
allowing much more customisation and targeted data extraction, this will be beneficial for those
who only want to extract very specific info, which will vastly speed up data extraction time.

## Usage

    sra_metadata_parser --file <file> --destination <destination>

## Example 

    sra_metadata_parser -f NCBI_SRA_Metadata_Full_20201006.tar.gz -d ./
    
    
# Building

###### Install Rust: https://www.rust-lang.org/tools/install

###### Clone this repo

    git clone https://github.com/jamespeterschinner/sra_metadata_parser
    
###### Build

    cargo build --release    
    
###### Executable will now be in

    ~/sra_metadata_parser/target/release/sra_metadata_pipeline
    
# Disclaimer 

This project is currently 'good enough' code, which in part is a proof of concept
aimed at solving a particular problem (extracting relevant sra metadata) in a timely
manner. In order to achieve this goal the use of `unsafe` code with pointers of `'static`
life time to a `Vec<'a, u8>` is used. Currently the implementation checks at runtime that
the capacity of this vec does not change, which would mean the unsafe pointers are now invalid.

I would like to change this in the future, providing there are not significant performance costs

