use fawkes_crypto::{
    backend::bellman_groth16::{engines::Bn256, verifier::VK},
    borsh::BorshSerialize,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let path = &args[1];
    let new_path = args.get(2).cloned().unwrap_or_else(|| path.replace(".json", ".bin"));

    println!("Reading {path}");
    let json = std::fs::read_to_string(path).unwrap();
    let vk: VK<Bn256> = serde_json::from_str(&json).unwrap();
    let vk_bin = vk.try_to_vec().unwrap();

    println!("Writing into {new_path}");
    std::fs::write(new_path, &vk_bin).unwrap();
}
