extern crate criterion;

use criterion::*;

use serde::{Deserialize, Serialize};
use serde_json;

use adblock;
use adblock::utils::rules_from_lists;
use adblock::blocker::{Blocker, BlockerOptions};
use adblock::request::Request;
use adblock::engine::Engine;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct TestRequest {
    frameUrl: String,
    url: String,
    cpt: String
}

fn load_requests() -> Vec<TestRequest> {
    let requests_str = adblock::utils::read_rules("data/requests.json");
    let reqs: Vec<TestRequest> = requests_str.into_iter().map(|r| serde_json::from_str(&r)).filter_map(Result::ok).collect();
    reqs
}

fn get_blocker(rules: &Vec<String>) -> Blocker {
  let (network_filters, _) = adblock::lists::parse_filters(rules, true, false, false);

  let blocker_options = BlockerOptions {
    debug: false,
    enable_optimizations: true,
    load_cosmetic_filters: false,
    load_network_filters: true
  };
  
  Blocker::new(network_filters, &blocker_options)
}

fn bench_rule_matching(blocker: &Blocker, requests: &Vec<TestRequest>) -> (u32, u32, u32) {
  let mut matches = 0;
  let mut passes = 0;
  let mut errors = 0;
  requests
    .iter()
    .for_each(|r| {
      let req: Result<Request, _> = Request::from_urls(&r.url, &r.frameUrl, &r.cpt);
      match req.map(|parsed| blocker.check(&parsed)).as_ref() {
        Ok(check) if check.matched => matches += 1,
        Ok(_) => passes += 1,
        Err(_) => errors += 1
      }
    });
  // println!("Got {} matches, {} passes, {} errors", matches, passes, errors);  
  (matches, passes, errors)
}

fn bench_matching_only(blocker: &Blocker, requests: &Vec<Request>) -> (u32, u32) {
  let mut matches = 0;
  let mut passes = 0;
  requests
    .iter()
    .for_each(|parsed| {
      let check =  blocker.check(&parsed);
      if check.matched {
        matches += 1;
      } else {
        passes += 1;
      }
    });
  println!("Got {} matches, {} passes", matches, passes);  
  (matches, passes)
}

fn rule_match(c: &mut Criterion) {
  
  let rules = rules_from_lists(&vec![
    String::from("data/easylist.to/easylist/easylist.txt"),
  ]);
  let requests = load_requests();
  let requests_len = requests.len() as u32;
  c.bench(
        "rule-match",
        Benchmark::new(
            "el",
            move |b| {
              let blocker = get_blocker(&rules);
              b.iter(|| bench_rule_matching(&blocker, &requests))
            },
        ).throughput(Throughput::Elements(requests_len))
        .sample_size(10)
    );
}

fn rule_match_elep(c: &mut Criterion) {
  
  let rules = rules_from_lists(&vec![
    String::from("data/easylist.to/easylist/easylist.txt"),
    String::from("data/easylist.to/easylist/easyprivacy.txt"),
  ]);
  let requests = load_requests();
  let requests_len = requests.len() as u32;
  c.bench(
        "rule-match",
        Benchmark::new(
            "el+ep",
            move |b| {
              let blocker = get_blocker(&rules);
              b.iter(|| bench_rule_matching(&blocker, &requests))
            },
        ).throughput(Throughput::Elements(requests_len))
        .sample_size(10)
    );
}

fn rule_match_slim(c: &mut Criterion) {
  let rules = rules_from_lists(&vec![
    String::from("data/slim-list.txt"),
  ]);
  let requests = load_requests();
  let requests_len = requests.len() as u32;
  
  c.bench(
        "rule-match",
        Benchmark::new(
            "slim",
            move |b| {
              let blocker = get_blocker(&rules);
              b.iter(|| bench_rule_matching(&blocker, &requests))
            },
        ).throughput(Throughput::Elements(requests_len))
        .sample_size(10)
    );
}

fn rule_match_only_el(c: &mut Criterion) {
  
  let rules = rules_from_lists(&vec![
    String::from("data/easylist.to/easylist/easylist.txt"),
  ]);
  let requests = load_requests();
  let requests_parsed: Vec<_> = requests.into_iter().map(|r| { Request::from_urls(&r.url, &r.frameUrl, &r.cpt) }).filter_map(Result::ok).collect();
  let requests_len = requests_parsed.len() as u32;
  let blocker = get_blocker(&rules);
  c.bench(
        "rule-match-parsed",
        Benchmark::new(
            "el",
            move |b| {
              b.iter(|| bench_matching_only(&blocker, &requests_parsed))
            },
        ).throughput(Throughput::Elements(requests_len))
        .sample_size(10)
    );
}

fn rule_match_slimlist_comparable(c: &mut Criterion) {
  
  let full_rules = rules_from_lists(&vec![
    String::from("data/easylist.to/easylist/easylist.txt"),
    String::from("data/easylist.to/easylist/easyprivacy.txt")
  ]);
  let blocker = get_blocker(&full_rules);
  
  let requests = load_requests();
  let requests_parsed: Vec<_> = requests.into_iter().map(|r| { Request::from_urls(&r.url, &r.frameUrl, &r.cpt) }).filter_map(Result::ok).collect();
  let requests_len = requests_parsed.len() as u32;

  let slim_rules = rules_from_lists(&vec![
    String::from("data/slim-list.txt"),
  ]);
  let slim_blocker = get_blocker(&slim_rules);

  let requests_copy = load_requests();
  let requests_parsed_copy: Vec<_> = requests_copy.into_iter().map(|r| { Request::from_urls(&r.url, &r.frameUrl, &r.cpt) }).filter_map(Result::ok).collect();

  c.bench(
        "rule-match-parsed",
        Benchmark::new(
            "el+ep",
            move |b| {
              b.iter(|| bench_matching_only(&blocker, &requests_parsed))
            },
        )
        .with_function("slimlist", move |b| {
              b.iter(|| bench_matching_only(&slim_blocker, &requests_parsed_copy))
            },)
        .throughput(Throughput::Elements(requests_len))
        .sample_size(10)
    );
}

fn serialization(c: &mut Criterion) {
  c.bench(
        "blocker-serialization",
        Benchmark::new(
            "el+ep",
            move |b| {
              let full_rules = rules_from_lists(&vec![
                String::from("data/easylist.to/easylist/easylist.txt"),
                String::from("data/easylist.to/easylist/easyprivacy.txt")
              ]);

              let engine = Engine::from_rules(&full_rules);
              b.iter(|| assert!(engine.serialize().unwrap().len() > 0) )
            },
        )
        .with_function(
          "el",
            move |b| {
              let full_rules = rules_from_lists(&vec![
                String::from("data/easylist.to/easylist/easylist.txt"),
              ]);

              let engine = Engine::from_rules(&full_rules);
              b.iter(|| assert!(engine.serialize().unwrap().len() > 0) )
            },)
        .with_function(
          "slimlist",
            move |b| {
              let full_rules = rules_from_lists(&vec![
                String::from("data/slim-list.txt"),
              ]);

              let engine = Engine::from_rules(&full_rules);
              b.iter(|| assert!(engine.serialize().unwrap().len() > 0) )
            },)
        .sample_size(20)
    );
}

fn deserialization(c: &mut Criterion) {
  c.bench(
        "blocker-deserialization",
        Benchmark::new(
            "el+ep",
            move |b| {
              let full_rules = rules_from_lists(&vec![
                String::from("data/easylist.to/easylist/easylist.txt"),
                String::from("data/easylist.to/easylist/easyprivacy.txt")
              ]);

              let engine = Engine::from_rules(&full_rules);
              let serialized = engine.serialize().unwrap();
              
              b.iter(|| {
                let mut deserialized = Engine::from_rules(&[]);
                assert!(deserialized.deserialize(&serialized).is_ok());
              })
            },
        )
        .with_function(
          "el",
            move |b| {
              let full_rules = rules_from_lists(&vec![
                String::from("data/easylist.to/easylist/easylist.txt"),
              ]);

              let engine = Engine::from_rules(&full_rules);
              let serialized = engine.serialize().unwrap();
              
              b.iter(|| {
                let mut deserialized = Engine::from_rules(&[]);
                assert!(deserialized.deserialize(&serialized).is_ok());
              })
            },)
        .with_function(
          "slimlist",
            move |b| {
              let full_rules = rules_from_lists(&vec![
                String::from("data/slim-list.txt"),
              ]);

              let engine = Engine::from_rules(&full_rules);
              let serialized = engine.serialize().unwrap();
              
              b.iter(|| {
                let mut deserialized = Engine::from_rules(&[]);
                assert!(deserialized.deserialize(&serialized).is_ok());
              })
            },)
        .sample_size(20)
    );
}

criterion_group!(
  benches,
  rule_match_only_el,
  rule_match_slimlist_comparable,
  rule_match,
  rule_match_elep,
  rule_match_slim,
  serialization,
  deserialization
);
criterion_main!(benches);
