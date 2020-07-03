use squeeze::lz77::Compressor;

fn main() {
  let file = std::fs::read_to_string("./src/lz77.rs").expect("Failed to read file");
  let compressor = Compressor::new();
  let comp: Vec<u8> = compressor.compress(&file.as_bytes(), 0);
  println!(
    "Compression ratio {}",
    comp.len() as f64 / file.len() as f64
  );
  let decomp: Vec<u8> = compressor.decompress(comp.as_slice());
  println!("Are equal: {:?}", file.as_bytes() == decomp.as_slice());
}
