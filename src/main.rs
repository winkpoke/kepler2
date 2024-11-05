use kepler_wgpu::{coordinates, run};

fn main() {
    let m = [1., 0.5, 0., 0., 
             0., 1., 0., 0., 
             0., 0., 1., 0., 
             0., 0., 0., 1.];
    let matrix = kepler_wgpu::coordinates::Matrix4x4::<f64>::from_array(m);
    println!("{:?}", matrix);
    println!("{:?}", matrix.apply(&[3., 2., 1., 1.]));
    let base0 = kepler_wgpu::coordinates::Base::<f64> {
        label: "world coordinate".to_string(),
        matrix: kepler_wgpu::coordinates::Matrix4x4::<f64>::eye(),
    };
    let base1 = kepler_wgpu::coordinates::Base::<f64> {
        label: "system coordinate".to_string(),
        matrix: matrix,
    };

    println!("{:?}", base0.to_base(&base1));
    pollster::block_on(run());
}
