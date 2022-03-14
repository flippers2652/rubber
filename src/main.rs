use vulkano::instance::{Instance, InstanceExtensions};
use vulkano::Version;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceExtensions, Features};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano::pipeline::ComputePipeline;
use vulkano::descriptor_set::PersistentDescriptorSet;
//use vulkano::pipeline::ComputePipelineAbstract;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::pipeline::Pipeline;
use vulkano::descriptor_set::WriteDescriptorSet;
use rand::Rng;
use std::fs::File;
use std::io::prelude::*;
mod cp {
    vulkano_shaders::shader! {
        ty: "compute",
        path:"src/shader.comp"
    }
}
fn run(){
    // Instance
    let instance = Instance::new(None, Version::V1_1, &InstanceExtensions::none(), None)
        .expect("failed to create instance");
    let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");

    // Device

    let queue_family = physical.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    let (device, mut queues) = {
        Device::new(physical, &vulkano::device::Features {
            shader_float64: true,

            .. Features::none()
        }, &DeviceExtensions::none(),
                    [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    };
    let queue = queues.next().unwrap();



    // Buffer
    #[allow(non_upper_case_globals)]
    const N :u32= 100;
    #[allow(non_upper_case_globals)]
    const nsim:u32=1000;
    #[allow(dead_code)]
    #[allow(non_snake_case)]
    #[derive(Clone,Copy)]
    struct Data {
        N:u32,
        nsim:u32,
    }
    let mut angle_data = [0_f64;(N*nsim) as usize];
    let length_data = [0_f64;(nsim) as usize];
    let mut rng = rand::thread_rng();
    for i in 0..nsim {
        angle_data[(i * N) as usize] = rng.gen_range(-std::f64::consts::FRAC_PI_2..std::f64::consts::FRAC_PI_2);
    }
    let n_data = [0_i32;(nsim) as usize];
    let n_buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, n_data)
        .expect("failed to create buffer");
    let angle_buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, angle_data)
        .expect("failed to create buffer");
    let init_buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, Data {N,nsim})
        .expect("failed to create buffer");
    let length_buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, length_data)
        .expect("failed to create buffer");
    // Pipeline
    let compute = cp::load(device.clone())
        .expect("failed to create shader module");

    let compute_pipeline = ComputePipeline::new(
        device.clone(),
        compute.entry_point("main").unwrap(),
        &(),
        None,
        |_|{}
    ).expect("failed to create compute pipeline");


    let layout = compute_pipeline
        .layout()
        .descriptor_set_layouts()
        .get(0)
        .unwrap();

    let set = PersistentDescriptorSet::new(layout.clone()
    ,[
                                               WriteDescriptorSet::buffer(1, angle_buffer.clone()),
                                           WriteDescriptorSet::buffer(2, n_buffer.clone()),
                                           WriteDescriptorSet::buffer(0, init_buffer),
                                           WriteDescriptorSet::buffer(3, length_buffer.clone())
                                           ],)

        .unwrap();
        //.unwrap();


    let mut builder = AutoCommandBufferBuilder::primary(
        device.clone(),
        queue.family(),
        CommandBufferUsage::OneTimeSubmit,
    )
        .unwrap();

    builder
        .bind_pipeline_compute(compute_pipeline.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            compute_pipeline.layout().clone(),
            0, // 0 is the index of our set
            set,
        )
        .dispatch([nsim, 1, 1])
        .unwrap();

    let command_buffer = builder.build().unwrap();
    let future = sync::now(device)
        .then_execute(queue, command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();
    future.wait(None).unwrap();

    let _content = angle_buffer.read().unwrap();
    let mut file = File::create("file.csv").unwrap();

    let content = n_buffer.read().unwrap();
    let ns = content.iter();
    let content = length_buffer.read().unwrap();
    let lengths = content.iter();
    let iter =std::iter::zip(ns,lengths);
    let mut sum=0.;
    for (&n,&l) in iter{
        writeln!(&mut file, "{n:04},{l:04},").unwrap();
        sum+=l*l;
        if n<0 {

            println!("{n:04},{l:04},");
        }
        //println!();
    }
    println!("{}",(sum/nsim as f64).sqrt());
    for &_f in angle_buffer.read().unwrap().iter(){
        //writeln!(&mut file, "{f:04},").unwrap();
        //println!();
    }
}
const STACK_SIZE: usize = 40000 * 1024 * 1024;
fn main() {
    // Spawn thread with explicit stack size
    let child = std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .unwrap();

    // Wait for thread to join
    child.join().unwrap();
}