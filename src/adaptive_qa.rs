use rand::Rng;

pub fn analyze_prompt(input: &str) -> (f32, f32) {
    let _ = input;
    let mut rng = rand::thread_rng();
    let drift: f32 = rng.gen_range(0.0..1.0);
    let res: f32 = rng.gen_range(0.0..1.0);
    (drift, res)
}
