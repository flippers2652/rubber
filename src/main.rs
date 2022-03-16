#![allow(clippy::needless_question_mark)]
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
    // Instance Creation
    let instance = Instance::new(None, Version::V1_1, &InstanceExtensions::none(), None)
        .expect("failed to create instance");
    let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");


    // Device Creation
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


    // Buffer Creation
    #[allow(non_upper_case_globals)]
    const nsimx:u32=200; // Number of simulations
    const nsimy:u32=200; // Number of simulations
    const nsimz:u32=200; // Number of simulations
    const N :u32= 10000; // Number of Edges per simulation

    #[allow(dead_code)] // Rust does not realise another language (GLSL) uses this bit of data
    #[allow(non_snake_case)]
    #[derive(Clone,Copy)] // Make this structure be a nice thing
    #[repr(C)] // So rust doesn't move things around as it thinks no one will notice (they will)
    struct Data {
        N:u32,
        nsimx:u32,
        nsimy:u32,
        nsimz:u32,
    }
    const nsim :u64=nsimx as u64*nsimy as u64*nsimz as u64;
    // I don't need all this data now but I did at one point so I'm keeping it
    let mut angle_data = [0_f64;(2*nsim) as usize];
    let length_data = [0_f64;(nsim) as usize];
    let n_data = [0_i32;(nsim) as usize];

    // Randomly Create the first angle in the sequence as G.P.U.s are bad at random numbers
    let mut rng = rand::thread_rng();
    for i in 0..nsim{
        angle_data[(i * 2) as usize] = rng.gen_range(-std::f64::consts::FRAC_PI_2..std::f64::consts::FRAC_PI_2);
    }

    let n_buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, n_data)
        .expect("failed to create buffer");
    let angle_buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, angle_data)
        .expect("failed to create buffer");
    let init_buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, Data {N,nsimx,nsimy,nsimz})
        .expect("failed to create buffer");
    let length_buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, length_data)
        .expect("failed to create buffer");


    // Pipeline
    // Ignore this. It just means please run the shader.comp
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
    //println!("{}",nsim%65535);
    builder
        .bind_pipeline_compute(compute_pipeline.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            compute_pipeline.layout().clone(),
            0, // 0 is the index of our set
            set,
        )
        .dispatch([nsimx as u32, nsimy as u32, nsimz as u32])
        .unwrap();

    let command_buffer = builder.build().unwrap();

    // Tell code to start
    let future = sync::now(device)
        .then_execute(queue, command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    // Wait for code to finish
    future.wait(None).unwrap();

    // Get buffers
    let content = n_buffer.read().unwrap();
    let ns = content.iter();
    let content = length_buffer.read().unwrap();
    let lengths = content.iter();
    let iter =std::iter::zip(ns,lengths);

    // For calculating R.M.S.
    let mut sum=0.;
    // Writes data to a csv file

    let mut file = File::create("file.csv").unwrap();
    for (&n,&l) in iter{
        writeln!(&mut file, "{n:04},{l:04},").unwrap();
        sum+=l*l;
    }

    // Prints R.M.S. of lengths
    println!("{}",(sum/(nsimx*nsimy*nsimz) as f64).sqrt());

    // Debug stuff
    /*
    for &_f in angle_buffer.read().unwrap().iter(){
        //writeln!(&mut file, "{f:04},").unwrap();
        //println!();
    }*/
}
const STACK_SIZE: usize = 40000 * 1024 * 1024;
fn main() {
    // Spawn thread with explicit stack size as main's is tiny
    let child = std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .unwrap();

    // Wait for thread to finish
    child.join().unwrap();
}
