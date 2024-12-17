use kepler_wgpu::{coord, run};

#[cfg(not(target_arch="wasm32"))]
#[tokio::main]
async fn main() {
    // pollster::block_on(run());
    run().await;
}
