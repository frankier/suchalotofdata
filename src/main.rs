#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate bincode;
extern crate itertools;
extern crate ordered_float;
extern crate seahash;
extern crate rand;
extern crate arrayvec;
extern crate csv;
extern crate num;

pub mod priority_queue;

use std::collections::{HashMap};
use std::fs::File;
use std::io::prelude::*;
use std::env;
use std::path::Path;
use std::fs;
use std::hash::Hasher;
use std::borrow::Borrow;
use rand::{random, thread_rng, Rng};
use std::time::Instant;
use std::f64::INFINITY;

use bincode::{Infinite, serialize_into, deserialize_from};
use itertools::{chain, Itertools};
use ordered_float::OrderedFloat;

use seahash::SeaHasher;

use priority_queue::{VecMaxSizePriorityQ, MaxSizePriorityQ};

use arrayvec::ArrayVec;
use num::clamp;

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
struct OnlyApproxKNNModel {
    hyperplanes: ArrayVec<HyperplaneEnsemble>,
    model: HashMap<HyperplaneHash, Vec<usize>>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct ApproxKNNModel {
    approx: OnlyApproxKNNModel,
    full: ExactKNNModel,
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

fn dataset_to_vec_streams(filepath: &Path)
        -> (Box<Iterator<Item=SparseVec>>, Box<Iterator<Item=SparseVec>>) {
    let ham_dir = filepath.join("ham");
    let spam_dir = filepath.join("spam");

    let ham_stream = convert_dir(&ham_dir).map(|ham_vec| {
        ham_vec
    });

    let spam_stream = convert_dir(&spam_dir).map(|spam_vec| {
        spam_vec
    });

    (Box::new(ham_stream), Box::new(spam_stream))
}

fn dataset_to_vec_stream(filepath: &Path)
        -> Box<Iterator<Item=(SparseVec, HamSpam)>> {
    let (ham_stream, spam_stream) = dataset_to_vec_streams(filepath);

    Box::new(chain(
        ham_stream.map(|v| {
            (v, HamSpam::Ham)
        }), spam_stream.map(|v| {
            (v, HamSpam::Spam)
        })))
}

fn make_knn_model(filepath: &Path) -> ExactKNNModel {
    dataset_to_vec_stream(filepath).collect_vec()
}

fn make_approx_knn_model(filepath: &Path) -> ApproxKNNModel {
    let full = make_knn_model(filepath);
    let approx = approxify(&full, 12);
    ApproxKNNModel { full, approx }
}

fn approxify(exact_model: &ExactKNNModel, num_hyperplanes: u8) -> OnlyApproxKNNModel {
    let mut hyperplanes = ArrayVec::<HyperplaneEnsemble>::new();
    for i in 0..num_hyperplanes {
        hyperplanes.insert(i as usize, Hyperplane {
            k1: rand::random::<u64>(),
            k2: rand::random::<u64>(),
            k3: rand::random::<u64>(),
            k4: rand::random::<u64>()
        });
    }

    let mut approx_model = HashMap::<HyperplaneHash, Vec<usize>>::new();
    for (idx, &(ref msg_vec, _)) in exact_model.iter().enumerate() {
        let hash = hyperplane_lsh(hyperplanes.as_slice(), msg_vec);
        if approx_model.contains_key(&hash) {
            approx_model.get_mut(&hash).unwrap().push(idx);
        } else {
            approx_model.insert(hash, vec![idx]);
        }
    }

    OnlyApproxKNNModel {
        hyperplanes: hyperplanes,
        model: approx_model,
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
    let cos_dist = (dot as f64) / ((mag_a as f64).sqrt() * (mag_b as f64).sqrt());
    clamp(cos_dist, -1.0, 1.0).acos()
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

fn hash_match<'a>(approx: &'a OnlyApproxKNNModel, query_vec: &SparseVec)
        -> Option<&'a Vec<usize>> {
    let query_hash = hyperplane_lsh(
        approx.hyperplanes.as_slice(),
        &query_vec);
    approx.model.get(&query_hash)
}

fn match_approx(model: &ApproxKNNModel, query: &Path) -> HamSpamUnk {
    let query_vec = convert_file(&query).unwrap();
    println!("Query vec {:?}", query_vec);
    println!("Hyperplanes {:?}", model.approx.hyperplanes);
    if let Some(idxs) = hash_match(&model.approx, &query_vec) {
        let mut ham = 0;
        let mut spam = 0;
        for idx in idxs {
            let (_, ref label) = model.full[*idx];
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

fn smallest_dist(model: &ExactKNNModel, query: &SparseVec) -> f64 {
    let mut min_d = INFINITY;
    for &(ref vec, _) in model.iter() {
        let d = arcdist(&query, vec);
        if d < min_d {
            min_d = d;
        }
    }
    min_d
}

fn task3(path: &Path) {
    let mut wtr = csv::Writer::from_writer(File::create("chart.csv").unwrap());
    wtr.write_record(&["num_hyperplanes", "secs", "nanos", "mean_abs_err"]).unwrap();
    // Load the messages from the first five datasets in memory
    let full = [
        "enron1",
        "enron2",
        "enron3",
        "enron4",
        "enron5"
    ].iter().flat_map(|model_dir| {
        dataset_to_vec_stream(&path.join(model_dir))
    }).collect_vec();
    // Load 100 messages (50 spam, 50 genuine) from the sixth dataset - the query messages
    let (ham_stream, spam_stream) = dataset_to_vec_streams(&path.join("enron6"));
    let query_msgs = chain(ham_stream.take(50), spam_stream.take(50)).collect_vec();
    println!("Data loaded. Starting experiments.");
    //For each query message find the distance of the exact nearest neighbor.
        //Record the time needed for this computation
    let mut exact_smallest_dists = ArrayVec::<[f64; 100]>::new();
    let nn_start = Instant::now();
    for query in query_msgs.iter() {
        exact_smallest_dists.push(smallest_dist(&full, query));
    }
    let nn_time = nn_start.elapsed();
    let secs = nn_time.as_secs();
    let nanos = nn_time.subsec_nanos() as u64;
    println!("Exact time {} {}", secs, nanos);
    println!("smallest_dists {:?}", exact_smallest_dists);
    wtr.serialize((0, secs, nanos, 0)).unwrap();
    //For numberOfHyperplanes in [1, 2, 4, 8, 16, 32]: #You can go up till 60/64 if it does not take too much time to compute
    for num_hyperplanes in [1, 2, 4, 8, 16, 32, 64].iter() {
        println!("{} hyperplanes", num_hyperplanes);
        //Create the data structure from Part II with numberOfHyperplanes hyper planes
        let approx_model = approxify(&full, *num_hyperplanes);
        let mut approx_smallest_dists = ArrayVec::<[f64; 100]>::new();
        //Start a timer
        let approx_nn_start = Instant::now();
        //For each query message
            //Find the result and distance for the approximate search
        for query in query_msgs.iter() {
            if let Some(idxs) = hash_match(&approx_model, &query) {
                let mut min_d = INFINITY;
                for idx in idxs.iter() {
                    let (ref vec, _) = full[*idx];
                    let d = arcdist(&query, vec);
                    if d < min_d {
                        min_d = d;
                    }
                }
                approx_smallest_dists.push(min_d);
            } else {
                let mut rng = thread_rng();
                let &(ref rand_vec, _) = rng.choose(&full).unwrap();
                let d = arcdist(&query, rand_vec);
                approx_smallest_dists.push(d);
            }
        }
        println!("smallest_dists {:?}", approx_smallest_dists);
        //Stop the timer
        let approx_nn_time = approx_nn_start.elapsed();
        let secs = approx_nn_time.as_secs();
        let nanos = approx_nn_time.subsec_nanos() as u64;
        println!("Approx time {} {}", secs, nanos);
        //Calculate the average error for the current number of hyperplanes formula for calculation of average absolute error
        let abs_err_sum: f64 = approx_smallest_dists.iter().zip(&exact_smallest_dists)
                .map(|(a, b)| {
            if a.is_infinite() {
                panic!("Didn't expect infinite distances")
            } else {
                a - b
            }
        }).sum();
        let avg_abs_err = abs_err_sum / 100.0;
        wtr.serialize((num_hyperplanes, secs, nanos, avg_abs_err)).unwrap();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_ref() {
        "make" => {
            if args.len() <= 3 {
                println!("Not enough args!");
                return;
            }
            let path_str: &str = args[2].as_ref();
            let path = Path::new(path_str);
            let knn_model = make_knn_model(path);
            let mut buffer = File::create(&args[3]).unwrap();
            serialize_into(&mut buffer, &knn_model, Infinite).unwrap();
        }
        "make_approx" => {
            if args.len() <= 3 {
                println!("Not enough args!");
                return;
            }
            let path_str: &str = args[2].as_ref();
            let path = Path::new(path_str);
            let knn_model = make_approx_knn_model(path);
            let mut buffer = File::create(&args[3]).unwrap();
            serialize_into(&mut buffer, &knn_model, Infinite).unwrap();
        }
        "match_exact" => {
            if args.len() <= 3 {
                println!("Not enough args!");
                return;
            }
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
            if args.len() <= 3 {
                println!("Not enough args!");
                return;
            }
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
        "task3" => {
            if args.len() <= 2 {
                println!("Not enough args!");
                return;
            }
            let path_str: &str = args[2].as_ref();
            let path = Path::new(path_str);
            task3(path);
        }
        _ => {
            println!("Unknown command {}", args[1]);
        }
    }
}
