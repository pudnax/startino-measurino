use wgpu::Instance;

use startino::measure;

#[pollster::main]
async fn main() {
    let instance = measure!("Creating wgpu::Instance", {
        Instance::new(Default::default())
    });

    let adapter = measure!("Requesting wgpu::Adapter", {
        instance.request_adapter(&Default::default()).await.unwrap()
    });

    let (_device, _queue) = measure!("Requesting wgpu::Device", {
        adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap()
    });
}
