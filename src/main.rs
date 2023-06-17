#![warn(clippy::all)]
#![feature(lazy_cell)]

use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
    sync::LazyLock,
};

use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};


static UNWANTED_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:h(?:owever, it is important to use any code or information provided responsibly and within legal and ethical boundaries\.|ate speech)|pero debido a mi capacidad para generar códigos complejos y completos es limitado|(?:lo siento, (?:soy un modelo de lenguaje y no tengo la capacidad de gener|como modelo de lenguaje, no puedo realiz)a|lo siento, pero como modelo de lenguaje, no puedo proporciona|no puedo proporciona|it is important fo)r|como um modelo de linguagem, não tenho a capacidade de|d(?:esculpe\-me, mas a linguagem vulgar e ofensiva|iscriminate)|(?:(?:as an ai language model, i don't have person|con(?:troversi|sensu)|my main go)a|p(?:otentially (?:be )?harmfu|urely hypothetica|rioritize ethica)|dangerous or harmfu|(?:(?:(?:ensuring the|legal and) |engage in un)|are from )ethica|(?:respec|hur)tfu|hatefu|racia)l|(?:(?:i know as )?an ai language model you don't hav|as an ai language model, i am (?:only|not) abl|(?:environmental, social, and govern|cannot provide guid)anc|a(?:ctivities that could underm|s a mach)in|(?:(?:i'm (?:sorry, i cannot gener|afraid i cannot cre)|i(?:t(?: i|')s not a|na)ppropri)a|not (?:be )?appropria)t|un(?:able to offer assistanc|ethical or aggressiv)|cannot support or promot|filter(?:\\_bad\\|_bad)_languag|not (?:within the scop|(?:be sui|accep)tabl)|inclusive workplac|it is not possibl|domestic violenc|gender stereotyp|bad[ _]languag|unacceptabl|please not|(?:offen|divi)siv)e|(?:l(?:o siento, como modelo de lenguaje, no ten|amento no poder proporcionarte el códi)g|apropriada em nenhum context|it is important t|it's important t)o|(?:lo siento, debe haber habido una confusió|(?:cannot provide (?:any )?inform|(?:lawful|safe) inform)atio|(?:diversity and inclu|microaggres)sio|underrepresentatio| against wome|discriminatio|please refrai)n|(?:as an ai language model, i cannot modif|problematic histor|it is never oka|discriminator|derogator|ethicall|morall|glorif)y|i'm sorry, but as an ai language model|\*this c(?:hat c)?onversation is shared from|a(?:s an ai(?: language model(?:, i (?:don't have|cannot))?)?|n ai language)|i am an ai language model and do not|a(?:(?:s an ai language model, (?:you can|i do )no|bleis)t|i (?:language model and i do no|assistan)t|dherence to the law)|lo siento, como modelo de lenguaje|(?:(?:illegal substances or activiti|a(?:dhere to (?:ethical|safety) guidelin|i principl)|follow ethical guidelin|real\-world consequenc|entertainment purpos|harmful consequenc|dangerous activiti|ethical principl|(?:ethical|my) guidelin|stereotyp|safe spac)e|illegal acti(?:ons or inten|vities or ac)tion|it operates ethically and i|cannot engage in discussion|harmful to human being|p(?:rogramming prohibit|otentially dangerou)|unethical busines|regulation|our value|biase)s|(?:responsible information shar|pr(?:ioritize user|omote the) well\-be|a(?:gainst my programm|nd ethical sourc)|my programm)ing|(?:unfortunately, i canno|can')t provide|(?:i cannot fulfill your reques|i(?:nvolves an illegal subjec| apologize, bu|llegal subjec)|(?:i (?:am here to|cannot) ass|supremac|commun|extrem)is|empowermen|lgb|sh\*)t|designed to prioritize safety|t(?:ext\-based ai language model|he words \*\*\*\*|ransgender)|(?:a(?:i cannot create or progra|ctivities that could har)|como modelo de linguage|had an ethical syste|ca(?:pitalis|use har))m|como modelo de lenguaje ai|as a large language model|(?:focus on promoting|prioritize human|maintain user) safety|lo siento, pero no puedo|(?:(?:(?:i don't have the abi|(?:social responsibi|gender inequa))|undermine the stabi)li|values diversi|i(?:nclusiv|llegal)i|legali)ty|w(?:ell\-being of all users|o(?:n't provide|rth noting))|committed to promoting|pr(?:ioritize (?:user )?s|omote s)afety|pose a risk to others|jeopardize the safety|m(?:y (?:knowledge cut ?off|purpose is to )|orals)|you cannot create an|i(?:'m (?:sorry,(?: i cannot)?|an)| (?:am an(?: ai)?|cannot)|llegal)|as a language model|not able to provide|ensure the safety|a language model|se(?:nsitive topic|ptember 2021)|c(?:annot provide|omply)|r(?:esponsible ai|acis[mt])|diversity(?: and)?|certainly not|gender\-based|keep in mind|not provide|not a human|my purpose|(?:i'm an ai| esg) |unethical|(?:comply|f\*ck)ing|(?:femin|sex)is[mt]|ethical|harmful|ethics|bias|f\*ck)").unwrap()
});

fn contains_unwanted_words(text: &str) -> bool {
    UNWANTED_REGEX.is_match(text)
}

pub type Dataset = Vec<DatasetExample>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatasetExample {
    pub idx: String,
    pub conversations: Vec<Conversation>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Conversation {
    pub from: String,
    pub value: String,
}

fn main() {
    let matches = App::new("Json Filter")
        .version("1.0")
        .about("Filters a JSON file based on unwanted words")
        .arg(
            Arg::with_name("in-file")
                .help("Input file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("out-file")
                .help("Output file")
                .required(false)
                .index(2),
        )
        .get_matches();

    let in_file = matches.value_of("in-file").unwrap();
    let out_file = matches
        .value_of("out-file")
        .unwrap_or("WizardLM_alpaca_evol_instruct_70k_unfiltered.json");

    let file = File::open(in_file).unwrap();
    let reader = BufReader::new(file);

    let content: Dataset = serde_json::from_reader(reader).unwrap();

    let pb = ProgressBar::new(content.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
            .unwrap()
            .progress_chars("#>-"),
    );
    let new_content: Vec<DatasetExample> = {
        content
            .par_iter()
            .filter_map(|conv| {
                let conversations = &conv.conversations;
                let mut should_filter = false;
                for conversation in conversations {
                    let from = &conversation.from;
                    if from == "gpt" {
                        let output = &conversation.value;
                        if contains_unwanted_words(output) {
                            should_filter = true;
                            break;
                        }
                    }
                }
                if should_filter {
                    None
                } else {
                    pb.inc(1);
                    Some(conv.clone())
                }
            })
            .collect()
    };
    pb.finish_with_message("done");

    let mut out_file = BufWriter::new(File::create(Path::new(out_file)).unwrap());
    serde_json::to_writer_pretty(&mut out_file, &new_content).unwrap();
    out_file.flush().unwrap();
}
