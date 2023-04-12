#![no_main]
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

#[derive(serde::Deserialize)]
struct Inputs {
    n: u32,
}

#[derive(serde::Serialize)]
struct Outputs {
    n: u32,
    sum: u64,
}

pub fn main() {
    let data: String = env::read();
    let inputs: Inputs = serde_json::from_str(&data).expect("should be deserializable");
    let n: u32 = inputs.n;
    let mut sum: u64 = 0;
    if n == 0 {
        panic!("invalid argument");
    } else if n == 1 {
        sum = 1;
    } else {
        let mut last = 0;
        let mut curr = 1;
        for _ in 1..n {
            sum = last + curr;
            last = curr;
            curr = sum;
        }
    }
    let result = serde_json::to_string(&Outputs { n, sum }).expect("should be serializable");
    env::commit(&result);
}
