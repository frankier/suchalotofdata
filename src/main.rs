#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate bincode;
extern crate itertools;
extern crate ordered_float;
extern crate seahash;
extern crate rand;
extern crate arrayvec;

pub mod priority_queue;

use std::collections::{HashMap};
use std::fs::File;
use std::io::prelude::*;
use std::env;
use std::path::Path;
use std::fs;
use std::hash::Hasher;
use std::borrow::Borrow;
use rand::random;

use bincode::{Infinite, serialize_into, deserialize_from};
use itertools::{chain, Itertools};
use ordered_float::OrderedFloat;

use seahash::SeaHasher;

use priority_queue::{VecMaxSizePriorityQ, MaxSizePriorityQ};

use arrayvec::ArrayVec;
use arrayvec::Array;

const NUM_HYPERPLANES: usize = 12;

type SparseVec = HashMap<String, u64>;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, PartialOrd, Ord)]
enum HamSpam {
    Ham,
    Spam,
}

enum HamSpamUnk {
    Ham,
    Spam,
    Unk,
}

type ExactKNNModel = Vec<(SparseVec, HamSpam)>;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Hyperplane {
    k1: u64,
    k2: u64,
    k3: u64,
    k4: u64,
}
type HyperplaneEnsemble = [Hyperplane; 64];

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
struct HyperplaneHash(u64);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct ApproxKNNModel {
    hyperplanes: ArrayVec<HyperplaneEnsemble>,
    model: HashMap<HyperplaneHash, Vec<HamSpam>>
}

fn hyperplane_lsh(hyperplanes: &[Hyperplane], vec: &SparseVec) -> HyperplaneHash {
    let mut lsh = 0;
    for (idx, hyperplane) in hyperplanes.iter().enumerate() {
        let mut dot: i64 = 0;
        for (k, &v) in vec.iter() {
            let mut hasher = SeaHasher::with_seeds(hyperplane.k1, hyperplane.k2, hyperplane.k3, hyperplane.k4);
            hasher.write(k.as_bytes());
            if hasher.finish() > <u64>::max_value() / 2 {
                dot += v as i64;
            } else {
                dot -= v as i64;
            }
        }
        if dot > 0 {
            lsh |= 1 << idx;
        }
    }
    HyperplaneHash(lsh)
}

fn convert(email: &str) -> SparseVec {
    let mut v = HashMap::<String, u64>::new();

    for word in email.split_whitespace() {
        if v.contains_key(word) {
            *v.get_mut(word).unwrap() += 1;
        } else {
            v.insert(word.to_owned(), 1);
        }
    };

    v
}

fn convert_file(path: &Path) -> Option<SparseVec> {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    if let Err(err) = file.read_to_string(&mut contents) {
        eprintln!("Warning error while reading {:?}: {}.", path, err);
        eprintln!("File discarded");
        None
    } else {
        Some(convert(contents.as_ref()))
    }
}

fn convert_dir(dirpath: &Path) -> Box<Iterator<Item=SparseVec>> {
    Box::new(fs::read_dir(dirpath).unwrap().filter_map(|filen| {
        convert_file(&filen.unwrap().path())
    }))
}

fn dataset_to_vec_stream(filepath: &Path)
        -> Box<Iterator<Item=(SparseVec, HamSpam)>> {
    let ham_dir = filepath.join("ham");
    let spam_dir = filepath.join("spam");

    let ham_stream = convert_dir(&ham_dir).map(|ham_vec| {
        (ham_vec, HamSpam::Ham)
    });

    let spam_stream = convert_dir(&spam_dir).map(|spam_vec| {
        (spam_vec, HamSpam::Spam)
    });

    Box::new(chain(ham_stream, spam_stream))
}

fn make_knn_model(filepath: &Path) -> ExactKNNModel {
    dataset_to_vec_stream(filepath).collect_vec()
}

fn make_approx_knn_model(filepath: &Path) -> ApproxKNNModel {
    let vec_stream = dataset_to_vec_stream(filepath);
    let mut hyperplanes = ArrayVec::<HyperplaneEnsemble>::new();
    for i in 0..NUM_HYPERPLANES {
        hyperplanes.insert(i, Hyperplane {
            k1: rand::random::<u64>(),
            k2: rand::random::<u64>(),
            k3: rand::random::<u64>(),
            k4: rand::random::<u64>()
        });
    }

    let mut model = HashMap::<HyperplaneHash, Vec<HamSpam>>::new();
    for (msg_vec, label) in vec_stream {
        let hash = hyperplane_lsh(hyperplanes.as_slice(), &msg_vec);
        if model.contains_key(&hash) {
            model.get_mut(&hash).unwrap().push(label);
        } else {
            model.insert(hash, vec![label]);
        }
    }

    ApproxKNNModel {
        hyperplanes: hyperplanes,
        model: model,
    }
}

fn arcdist(a: &SparseVec, b: &SparseVec) -> f64 {
    let mut dot: u64 = 0;
    let mut mag_a: u64 = 0;
    for (k_a, v_a) in a.iter() {
        if let Some(v_b) = b.get(k_a) {
            dot += v_a * v_b;
        }
        mag_a += v_a * v_a;
    }
    let mut mag_b: u64 = 0;
    for v_b in b.values() {
        mag_b += v_b * v_b;
    }
    //println!("dot: {}, mag_a: {}, mag_b: {}", dot, mag_a, mag_b);
    let cos_dist = (dot as f64) / ((mag_a as f64).sqrt() * (mag_b as f64).sqrt());
    if cos_dist > 1.0 || cos_dist < -1.0 {

        //println!("cos_dist: {}", cos_dist);
    }
    cos_dist.acos()
}

fn knn_exact(model: &ExactKNNModel, query: SparseVec, k: u32) -> HamSpamUnk {
    println!("Query vector:");
    println!("{:?}", query);
    let mut q = VecMaxSizePriorityQ::new(k);

    for (idx, &(ref vec, _)) in model.iter().enumerate() {
        let d = arcdist(&query, vec);
        //println!("{} {}", idx, d);
        q.push((OrderedFloat(d), idx));
    }

    let mut ham = 0;
    let mut spam = 0;
    for &(dist, idx) in q.queue.iter() {
        let &(ref vec, ref label) = &model[idx];
        println!("Nearby (dist={}) vector has label {:?}:", dist.0, label);
        println!("{:?}", vec);
        match label {
            &HamSpam::Ham => ham += 1,
            &HamSpam::Spam => spam += 1,
        }
    }
    if ham > spam {
        HamSpamUnk::Ham
    } else if spam > ham {
        HamSpamUnk::Spam
    } else {
        HamSpamUnk::Unk
    }
}

fn match_exact(model: &ExactKNNModel, query: &Path) -> HamSpamUnk {
    let query_vec = convert_file(&query).unwrap();
    knn_exact(model, query_vec, 3)
}

fn match_approx(model: &ApproxKNNModel, query: &Path) -> HamSpamUnk {
    let query_vec = convert_file(&query).unwrap();
    println!("Query vec {:?}", query_vec);
    println!("Hyperplanes {:?}", model.hyperplanes);
    let query_hash = hyperplane_lsh(
        model.hyperplanes.as_slice(),
        &query_vec);
    println!("Query hash {:?}", query_hash);
    if let Some(labels) = model.model.get(&query_hash) {
        let mut ham = 0;
        let mut spam = 0;
        for label in labels {
            match label {
                &HamSpam::Ham => {
                    ham += 1;
                    println!("Ham!");
                }
                &HamSpam::Spam => {
                    spam += 1;
                    println!("Spam!");
                }
            }
        }
        if ham > spam {
            HamSpamUnk::Ham
        } else if spam > ham {
            HamSpamUnk::Spam
        } else {
            HamSpamUnk::Unk
        }
    } else {
        println!("No match!");
        HamSpamUnk::Unk
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 3 {
        println!("Not enough args!");
        return;
    }
    match args[1].as_ref() {
        "make" => {
            let path_str: &str = args[2].as_ref();
            let path = Path::new(path_str);
            let knn_model = make_knn_model(path);
            let mut buffer = File::create(&args[3]).unwrap();
            serialize_into(&mut buffer, &knn_model, Infinite).unwrap();
        }
        "make_approx" => {
            let path_str: &str = args[2].as_ref();
            let path = Path::new(path_str);
            let knn_model = make_approx_knn_model(path);
            let mut buffer = File::create(&args[3]).unwrap();
            serialize_into(&mut buffer, &knn_model, Infinite).unwrap();
        }
        "match_exact" => {
            let mut buffer = File::open(&args[3]).unwrap();
            let knn_model = deserialize_from(&mut buffer, Infinite).unwrap();
            let result = match_exact(&knn_model, args[2].as_ref());
            match result {
                HamSpamUnk::Ham => {
                    println!("Ham!");
                }
                HamSpamUnk::Spam => {
                    println!("Spam!");
                }
                HamSpamUnk::Unk => {
                    println!("Unknown!");
                }
            }
        }
        "match_approx" => {
            let mut buffer = File::open(&args[3]).unwrap();
            let knn_model = deserialize_from(&mut buffer, Infinite).unwrap();
            let result = match_approx(&knn_model, args[2].as_ref());
            match result {
                HamSpamUnk::Ham => {
                    println!("Ham!");
                }
                HamSpamUnk::Spam => {
                    println!("Spam!");
                }
                HamSpamUnk::Unk => {
                    println!("Unknown!");
                }
            }
        }
        _ => {
            println!("Unknown command {}", args[1]);
        }
    }
}
